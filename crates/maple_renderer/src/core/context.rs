use super::{LazyBufferable, texture};
use crate::core::{Frame, RenderDevice, RenderQueue};
use crate::platform::SendSync;
use crate::types::Dimensions;
use crate::{
    core::{
        buffer::Buffer,
        descriptor_set::{DescriptorSetLayout, DescriptorSetLayoutDescriptor},
        mipmap_generator::{self, MipmapGenerator},
        texture::{LazyTexture, Texture, TextureCube, TextureView},
    },
    render_graph::node::RenderTarget,
    types::{
        default_texture::DefaultTexture,
        render_config::{RenderConfig, VsyncMode},
    },
};
use anyhow::Result;
use parking_lot::RwLock;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::collections::HashMap;
use std::{
    error::Error,
    sync::{Arc, OnceLock},
};
use wgpu::{
    Adapter, Device, DeviceDescriptor, Instance, InstanceDescriptor, PresentMode, Queue,
    RequestAdapterOptions, Surface, SurfaceConfiguration, SurfaceTexture, TextureFormat,
    TextureUsages,
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
}

/// Public rendering context that provides a safe API over the backend
pub struct RenderContext {
    backend: Backend,
    layout_cache: RwLock<HashMap<DescriptorSetLayoutDescriptor, DescriptorSetLayout>>,
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

    pub fn create_frame(&self) -> Frame<'_> {
        let encoder = self
            .device
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame command encoder"),
            });

        Frame {
            encoder: encoder,
            renderer: self,
        }
    }

    pub fn submit_frame(&self, frame: Frame<'_>) {
        self.queue
            .queue
            .submit(std::iter::once(frame.encoder.finish()));
    }

    pub fn attach_surface<T>(&mut self, window: Arc<T>, dimensions: Dimensions) -> Result<()>
    where
        T: HasDisplayHandle + HasWindowHandle + SendSync + 'static,
    {
        self.backend.attach_surface(window, dimensions)
    }

    pub fn get_surface_texture(&self) -> Option<&SurfaceTexture> {
        self.backend.get_surface_texture()
    }

    pub fn get_or_create_layout(
        &self,
        descriptor: DescriptorSetLayoutDescriptor,
    ) -> DescriptorSetLayout {
        {
            let cache = self.layout_cache.read();
            if let Some(layout) = cache.get(&descriptor) {
                return layout.clone();
            }
        }

        let layout = self.device.create_descriptor_set_layout(descriptor);
        self.layout_cache.write().insert(descriptor, layout.clone());
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
