use super::{LazyBufferable, texture};
use crate::core::{RenderDevice, RenderQueue};
use crate::platform::SendSync;
use crate::types::Dimensions;
use crate::{
    core::{
        ComputeBuilder, ComputeShader, ComputeShaderSource, DescriptorSetBuilder, GraphicsShader,
        ShaderPair,
        buffer::{Buffer, LazyBuffer},
        descriptor_set::{DescriptorSet, DescriptorSetLayout, DescriptorSetLayoutDescriptor},
        frame_builder::FrameBuilder,
        mipmap_generator::{self, MipmapGenerator},
        pipeline::{
            ComputePipeline, ComputePipelineCreateInfo, PipelineCreateInfo, PipelineLayout,
            RenderPipeline,
        },
        texture::{
            LazyTexture, Sampler, SamplerOptions, Texture, TextureCreateInfo, TextureCube,
            TextureCubeCreateInfo, TextureView,
        },
    },
    render_graph::node::RenderTarget,
    types::{
        Vertex,
        default_texture::DefaultTexture,
        render_config::{RenderConfig, VsyncMode},
    },
};
use anyhow::Result;
use bytemuck::Pod;
use parking_lot::RwLock;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::collections::HashMap;
use std::{
    error::Error,
    sync::{Arc, OnceLock},
};
use wgpu::{
    Adapter, BufferUsages, Device, DeviceDescriptor, Instance, InstanceDescriptor, Operations,
    PresentMode, Queue, RenderPassDepthStencilAttachment, RequestAdapterOptions, Surface,
    SurfaceConfiguration, SurfaceTexture, TextureFormat, TextureUsages,
};

pub struct RenderOptions<'a> {
    pub label: Option<&'a str>,
    pub color_targets: &'a [RenderTarget],
    pub depth_target: Option<&'a TextureView>,
    pub clear_color: Option<[f32; 4]>,
    pub clear_depth: Option<f32>,
}

/// holds all raw WGPU state
struct Backend {
    instance: Instance,
    adapter: Adapter,
    device: Arc<Device>,
    queue: Arc<Queue>,
    surface: Option<Surface<'static>>,
    current_surface_texture: Option<SurfaceTexture>,
    surface_format: texture::TextureFormat,
    config: RenderConfig,
    dimensions: Dimensions,

    default_textures: OnceLock<DefaultTexture>,
    mipmap_generator: MipmapGenerator,
}

impl Backend {
    async fn init<T>(window: Arc<T>, config: RenderConfig) -> Result<Self>
    where
        T: HasDisplayHandle + HasWindowHandle + SendSync + 'static,
    {
        let instance = Instance::new(&InstanceDescriptor::default());

        let adapter = instance
            .request_adapter(&RequestAdapterOptions::default())
            .await?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                required_features: wgpu::Features::TEXTURE_FORMAT_16BIT_NORM,
                ..Default::default()
            })
            .await?;

        let surface: Surface = instance.create_surface(window)?;
        let cap = surface.get_capabilities(&adapter);
        let surface_format: texture::TextureFormat = cap.formats[0].into();

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let mipmap_generator = MipmapGenerator::new(device.clone(), queue.clone());

        let backend = Self {
            instance: instance,
            adapter,
            device: device,
            queue: queue,
            surface: Some(surface),
            current_surface_texture: None,
            surface_format,
            config,
            dimensions: Dimensions::zero(),
            default_textures: OnceLock::new(),
            mipmap_generator,
        };

        backend.configure_surface();

        Ok(backend)
    }

    async fn init_headless(config: RenderConfig) -> Result<Self> {
        let instance = Instance::new(&InstanceDescriptor::default());

        let adapter = instance
            .request_adapter(&RequestAdapterOptions::default())
            .await?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                required_features: wgpu::Features::TEXTURE_FORMAT_16BIT_NORM,
                ..Default::default()
            })
            .await?;

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let mipmap_generator = MipmapGenerator::new(device.clone(), queue.clone());

        let backend = Self {
            instance: instance,
            adapter,
            device: device,
            queue: queue,
            surface: None,
            current_surface_texture: None,
            surface_format: texture::TextureFormat::BGRA8Srgb,
            config,
            dimensions: Dimensions::zero(),
            default_textures: OnceLock::new(),
            mipmap_generator,
        };

        Ok(backend)
    }

    fn attach_surface<T>(&mut self, window: Arc<T>, dimensions: Dimensions) -> Result<()>
    where
        T: HasDisplayHandle + HasWindowHandle + SendSync + 'static,
    {
        let surface: Surface = self.instance.create_surface(window)?;
        let cap = surface.get_capabilities(&self.adapter);
        self.surface_format = cap.formats[0].into();
        self.surface = Some(surface);
        self.dimensions = dimensions;
        self.configure_surface();
        Ok(())
    }

    fn configure_surface(&self) {
        let Some(surface) = self.surface.as_ref() else {
            return;
        };
        let format: TextureFormat = self.surface_format.into();

        surface.configure(
            &self.device,
            &SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format,
                view_formats: vec![format.add_srgb_suffix()],
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                width: self.dimensions.width,
                height: self.dimensions.height,
                desired_maximum_frame_latency: 2,
                present_mode: match self.config.vsync {
                    VsyncMode::Off => PresentMode::AutoNoVsync,
                    VsyncMode::On => PresentMode::AutoVsync,
                },
            },
        );
    }

    pub fn acquire_surface_texture(&mut self) -> Result<&SurfaceTexture, Box<dyn Error>> {
        if self.current_surface_texture.is_none() {
            let surface = self.surface.as_ref().expect("surface not attached");
            self.current_surface_texture = Some(surface.get_current_texture()?);
        }
        Ok(self.current_surface_texture.as_ref().unwrap())
    }

    pub fn get_surface_texture(&self) -> Option<&SurfaceTexture> {
        self.current_surface_texture.as_ref()
    }

    pub fn present_surface(&mut self) -> Result<(), Box<dyn Error>> {
        if let Some(surface_tex) = self.current_surface_texture.take() {
            surface_tex.present();
        }
        Ok(())
    }

    pub fn resize(&mut self, new_size: Dimensions) {
        self.dimensions = new_size;

        self.configure_surface();
    }

    pub fn change_vsync(&mut self, mode: VsyncMode) {
        self.config.vsync = mode;

        self.configure_surface();
    }

    pub fn render<F>(&self, options: RenderOptions, execute: F) -> Result<()>
    where
        F: FnOnce(FrameBuilder),
    {
        // Prepare the render target only as needed
        struct PreparedTarget {
            view: wgpu::TextureView,
            resolve_view: Option<wgpu::TextureView>,
        }

        let mut prepared = Vec::new();

        for target in options.color_targets {
            match target {
                RenderTarget::Surface => {
                    let surface_tex = self.get_surface_texture().unwrap();
                    let view = surface_tex
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    prepared.push(PreparedTarget {
                        view,
                        resolve_view: None,
                    });
                }
                RenderTarget::Texture(t) => {
                    prepared.push(PreparedTarget {
                        view: t.inner.clone(),
                        resolve_view: None,
                    });
                }
                RenderTarget::MultiSampled { texture, resolve } => prepared.push(PreparedTarget {
                    view: texture.inner.clone(),
                    resolve_view: Some(resolve.inner.clone()),
                }),
            }
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        {
            let depth_view = options.depth_target;

            let depth_stencil_attachment =
                depth_view
                    .as_ref()
                    .map(|view| RenderPassDepthStencilAttachment {
                        view: &view.inner,
                        depth_ops: Some(Operations {
                            load: options
                                .clear_depth
                                .map(wgpu::LoadOp::Clear)
                                .unwrap_or(wgpu::LoadOp::Load),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    });

            let color_attachments: Vec<Option<wgpu::RenderPassColorAttachment>> = prepared
                .iter()
                .map(|prepared_target| {
                    Some(wgpu::RenderPassColorAttachment {
                        view: &prepared_target.view,
                        resolve_target: prepared_target.resolve_view.as_ref(),
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: match options.clear_color {
                                Some([r, g, b, a]) => wgpu::LoadOp::Clear(wgpu::Color {
                                    r: r as f64,
                                    g: g as f64,
                                    b: b as f64,
                                    a: a as f64,
                                }),
                                None => wgpu::LoadOp::Load,
                            },
                            store: wgpu::StoreOp::Store,
                        },
                    })
                })
                .collect();

            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: options.label,
                color_attachments: &color_attachments,
                depth_stencil_attachment,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            let frame_builder = FrameBuilder::new(render_pass);
            // where we build the user command buffer pass in bound
            // automatically by frame builder
            execute(frame_builder);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        // done rendering this pass

        Ok(())
    }

    fn compute<F>(&self, label: Option<&str>, execute: F)
    where
        F: FnOnce(ComputeBuilder),
    {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            });
        {
            let compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label,
                timestamp_writes: None,
            });

            let compute_builder = ComputeBuilder::new(compute_pass);
            execute(compute_builder);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
    }
}

pub struct PipelineCache {}

/// Public rendering context that provides a safe API over the backend
pub struct RenderContext {
    backend: Backend,
    layout_cache: RwLock<HashMap<&'static str, DescriptorSetLayout>>,
    device: RenderDevice,
    queue: RenderQueue,
}

impl RenderContext {
    pub async fn init<T>(window: Arc<T>, config: RenderConfig) -> Result<Self>
    where
        T: HasDisplayHandle + HasWindowHandle + SendSync + 'static,
    {
        let backend = Backend::init(window, config).await?;
        Ok(Self {
            layout_cache: RwLock::new(HashMap::new()),
            device: RenderDevice {
                device: backend.device.clone(),
                queue: backend.queue.clone(),
            },
            queue: RenderQueue {
                queue: backend.queue.clone(),
            },
            backend,
        })
    }

    pub async fn init_headless(config: RenderConfig) -> Result<Self> {
        let backend = Backend::init_headless(config).await?;
        Ok(Self {
            layout_cache: RwLock::new(HashMap::new()),
            device: RenderDevice {
                device: backend.device.clone(),
                queue: backend.queue.clone(),
            },
            queue: RenderQueue {
                queue: backend.queue.clone(),
            },
            backend,
        })
    }

    pub fn attach_surface<T>(&mut self, window: Arc<T>, dimensions: Dimensions) -> Result<()>
    where
        T: HasDisplayHandle + HasWindowHandle + SendSync + 'static,
    {
        self.backend.attach_surface(window, dimensions)
    }

    pub fn get_or_create_layout(
        &self,
        key: &'static str,
        descriptor: DescriptorSetLayoutDescriptor,
    ) -> DescriptorSetLayout {
        {
            let cache = self.layout_cache.read();
            if let Some(layout) = cache.get(key) {
                return layout.clone();
            }
        }

        let layout = self.device.create_descriptor_set_layout(descriptor);
        self.layout_cache.write().insert(key, layout.clone());
        layout
    }

    pub fn device(&self) -> &RenderDevice {
        &self.device
    }

    pub fn queue(&self) -> &RenderQueue {
        &self.queue
    }

    pub fn mipmap_generator(&self) -> &MipmapGenerator {
        &self.backend.mipmap_generator
    }

    pub fn surface_format(&self) -> texture::TextureFormat {
        self.backend.surface_format
    }

    pub fn resize(&mut self, new_size: Dimensions) {
        self.backend.resize(new_size);
    }

    pub fn change_vsync(&mut self, mode: VsyncMode) {
        self.backend.change_vsync(mode);
    }

    pub fn acquire_surface_texture(&mut self) -> Result<&SurfaceTexture, Box<dyn Error>> {
        self.backend.acquire_surface_texture()
    }

    pub fn present_surface(&mut self) -> Result<(), Box<dyn Error>> {
        self.backend.present_surface()
    }

    pub fn surface_size(&self) -> Dimensions {
        self.backend.dimensions
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.backend.dimensions.width as f32 / self.backend.dimensions.height.max(1) as f32
    }

    pub fn sync_lazy_buffer<T, B>(&self, lazy_buffer: &B)
    where
        B: LazyBufferable<T>,
        T: ?Sized + SendSync,
    {
        lazy_buffer.sync(&self.backend.queue)
    }

    pub fn get_buffer<T, B>(&self, lazy_buffer: &B) -> Buffer<T>
    where
        B: LazyBufferable<T>,
        T: ?Sized + SendSync,
    {
        lazy_buffer.get_buffer(&self.backend.device, &self.backend.queue)
    }

    pub fn get_default_texture(&self) -> &DefaultTexture {
        self.backend.default_textures.get_or_init(|| {
            DefaultTexture::init_textures(&self.backend.device, &self.backend.queue)
        })
    }

    pub fn get_texture(&self, lazy_texture: &LazyTexture) -> Texture {
        lazy_texture.get_texture(
            &self.backend.mipmap_generator,
            &self.backend.device,
            &self.backend.queue,
        )
    }

    pub fn render<F>(&self, options: RenderOptions, execute: F) -> Result<()>
    where
        F: FnOnce(FrameBuilder),
    {
        self.backend.render(options, execute)
    }

    pub fn compute<F>(&self, label: Option<&str>, execute: F)
    where
        F: FnOnce(ComputeBuilder),
    {
        self.backend.compute(label, execute);
    }

    pub fn generate_mipmaps(&self, texture: &Texture, mip_level_count: u32) {
        mipmap_generator::generate_mipmaps(
            &self.backend.mipmap_generator,
            &self.backend.device,
            &self.backend.queue,
            &texture.inner,
            mip_level_count,
        );
    }

    pub fn generate_cubemap_mipmaps(&self, cubemap: &TextureCube, mip_level_count: u32) {
        mipmap_generator::generate_cubemap_mipmaps(
            &self.backend.mipmap_generator,
            &self.backend.device,
            &self.backend.queue,
            &cubemap.inner,
            mip_level_count,
        );
    }
}
