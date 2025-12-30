use std::{error::Error, sync::Arc};

use anyhow::Result;
use bytemuck::Pod;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use wgpu::{
    BufferUsages, Device, DeviceDescriptor, Instance, InstanceDescriptor, Operations, PresentMode,
    Queue, RenderPassDepthStencilAttachment, RequestAdapterOptions, Surface, SurfaceConfiguration,
    SurfaceTexture, TextureFormat, TextureUsages,
};

use crate::{
    core::{
        DescriptorSetBuilder, GraphicsShader, ShaderPair,
        buffer::{Buffer, LazyBuffer},
        descriptor_set::{DescriptorSet, DescriptorSetLayout, DescriptorSetLayoutDescriptor},
        frame_builder::FrameBuilder,
        pipeline::{PipelineCreateInfo, PipelineLayout, RenderPipeline},
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

use super::{LazyBufferable, texture};

pub struct RenderOptions<'a> {
    pub color_targets: &'a [RenderTarget],
    pub depth_target: Option<&'a TextureView>,
    pub clear_color: Option<[f32; 4]>,
}

/// holds all raw WGPU state
struct Backend {
    _instance: Instance,
    device: Device,
    queue: Queue,
    surface: Surface<'static>,
    current_surface_texture: Option<SurfaceTexture>,
    surface_format: texture::TextureFormat,
    config: RenderConfig,
    dimensions: (u32, u32),
}

impl Backend {
    async fn init<T>(window: Arc<T>, config: RenderConfig) -> Result<Self>
    where
        T: HasDisplayHandle + HasWindowHandle + Send + Sync + 'static,
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

        let dimensions = (config.dimensions[0], config.dimensions[1]);

        let backend = Self {
            _instance: instance,
            device,
            queue,
            surface,
            current_surface_texture: None,
            surface_format,
            config,
            dimensions,
        };

        backend.configure_surface();

        Ok(backend)
    }

    fn configure_surface(&self) {
        let format: TextureFormat = self.surface_format.into();

        self.surface.configure(
            &self.device,
            &SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format,
                view_formats: vec![format.add_srgb_suffix()],
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                width: self.config.dimensions[0],
                height: self.config.dimensions[1],
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
            let surface_tex = self.surface.get_current_texture()?;
            self.current_surface_texture = Some(surface_tex)
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

    pub fn resize(&mut self, new_size: [u32; 2]) {
        self.config.dimensions = new_size;
        self.dimensions = (new_size[0], new_size[1]);

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
                            load: wgpu::LoadOp::Clear(1.0),
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
                label: Some("Render Pass"),
                color_attachments: &color_attachments,
                depth_stencil_attachment,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Only nodes with render targets should call render()
            // Resource-only nodes (with no pipeline) should only use draw() for resource management
            // let pipeline = ctx.pipeline().expect("Cannot render with a resource-only node that has no pipeline. This node should not call render_ctx.render().");
            // render_pass.set_pipeline(&pipeline.backend);

            let frame_builder = FrameBuilder::new(render_pass);
            // where we build the user command buffer pass in bound
            // automatically by frame builder
            execute(frame_builder);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        // done rendering this pass

        Ok(())
    }
}

/// Public rendering context that provides a safe API over the backend
pub struct RenderContext {
    backend: Backend,
}

impl RenderContext {
    pub async fn init<T>(window: Arc<T>, config: RenderConfig) -> Result<Self>
    where
        T: HasDisplayHandle + HasWindowHandle + Send + Sync + 'static,
    {
        let backend = Backend::init(window, config).await?;
        Ok(Self { backend })
    }

    pub fn surface_format(&self) -> texture::TextureFormat {
        self.backend.surface_format
    }

    pub fn resize(&mut self, new_size: [u32; 2]) {
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

    pub fn surface_size(&self) -> (u32, u32) {
        self.backend.dimensions
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.backend.dimensions.0 as f32 / self.backend.dimensions.1.max(1) as f32
    }

    pub fn create_vertex_buffer(&self, vertices: &[Vertex]) -> Buffer<[Vertex]> {
        Buffer::from_slice(
            &self.backend.device,
            vertices,
            BufferUsages::VERTEX,
            "Vertex Buffer",
        )
    }

    pub fn create_index_buffer(&self, indices: &[u32]) -> Buffer<[u32]> {
        Buffer::from_slice(
            &self.backend.device,
            indices,
            BufferUsages::INDEX,
            "Index Buffer",
        )
    }

    pub fn create_uniform_buffer<T: Pod>(&self, uniform: &T) -> Buffer<T> {
        Buffer::from(
            &self.backend.device,
            uniform,
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            "Uniform Buffer",
        )
    }

    pub fn create_storage_buffer<T: Pod>(&self, data: &T) -> Buffer<T> {
        Buffer::from(
            &self.backend.device,
            data,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            "Storage Buffer",
        )
    }

    pub fn create_empty_storage_buffer<T: Pod>(&self) -> Buffer<T> {
        Buffer::empty(
            &self.backend.device,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            "storage buffer",
        )
    }

    pub fn create_storage_buffer_slice<T: Pod>(&self, data: &[T]) -> Buffer<[T]> {
        Buffer::from_slice(
            &self.backend.device,
            data,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            "Storage Buffer",
        )
    }

    pub fn create_sized_storage_buffer<T: Pod>(&self, len: usize) -> Buffer<[T]> {
        Buffer::from_size(
            &self.backend.device,
            len,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            "storage buffer",
        )
    }

    pub fn sync_lazy_buffer<T, B>(&self, lazy_buffer: &B)
    where
        B: LazyBufferable<T>,
        T: ?Sized,
    {
        lazy_buffer.sync(&self.backend.queue)
    }

    pub fn get_buffer<T, B>(&self, lazy_buffer: &B) -> Buffer<T>
    where
        B: LazyBufferable<T>,
        T: ?Sized,
    {
        lazy_buffer.get_buffer(&self.backend.device, &self.backend.queue)
    }

    pub fn write_buffer<T: Pod>(&self, buffer: &Buffer<T>, value: &T) {
        buffer.write(&self.backend.queue, value)
    }

    pub fn write_buffer_slice<T: Pod>(&self, buffer: &Buffer<[T]>, data: &[T]) {
        buffer.write(&self.backend.queue, data)
    }

    pub fn create_texture(&self, info: TextureCreateInfo) -> Texture {
        Texture::create(&self.backend.device, &info)
    }

    pub fn create_texture_cube(&self, info: TextureCubeCreateInfo) -> TextureCube {
        TextureCube::create(&self.backend.device, &info)
    }

    pub fn create_texture_array(
        &self,
        info: texture::TextureArrayCreateInfo,
    ) -> texture::TextureArray {
        texture::TextureArray::create(&self.backend.device, &info)
    }

    pub fn create_texture_cube_array(
        &self,
        info: texture::TextureCubeArrayCreateInfo,
    ) -> texture::TextureCubeArray {
        texture::TextureCubeArray::create(&self.backend.device, &info)
    }

    pub fn get_default_texture(&self) -> &DefaultTexture {
        DefaultTexture::get(&self.backend.device, &self.backend.queue)
    }

    pub fn create_lazy_texture(data: Vec<u8>, info: TextureCreateInfo) -> LazyTexture {
        LazyTexture::new(data, info)
    }

    pub fn get_texture(&self, lazy_texture: &LazyTexture) -> Texture {
        lazy_texture.get_texture(&self.backend.device, &self.backend.queue)
    }

    pub fn create_sampler(&self, options: SamplerOptions) -> Sampler {
        Texture::create_sampler(&self.backend.device, options)
    }

    pub fn write_texture(&self, texture: &Texture, data: &[u8]) {
        texture.write(&self.backend.queue, data)
    }

    pub fn load_texture_from_bytes(
        &self,
        bytes: &[u8],
        label: Option<&'static str>,
    ) -> Result<Texture, image::ImageError> {
        Texture::from_bytes(&self.backend.device, &self.backend.queue, bytes, label)
    }

    pub fn load_texture_from_file(
        &self,
        path: impl AsRef<std::path::Path>,
        label: Option<&'static str>,
    ) -> Result<Texture, image::ImageError> {
        Texture::from_file(&self.backend.device, &self.backend.queue, path, label)
    }

    pub fn load_lazy_texture_from_bytes(
        bytes: &[u8],
        label: Option<&'static str>,
    ) -> Result<LazyTexture, image::ImageError> {
        LazyTexture::from_bytes(bytes, label)
    }

    pub fn load_lazy_texture_from_file(
        path: impl AsRef<std::path::Path>,
        label: Option<&'static str>,
    ) -> Result<LazyTexture, image::ImageError> {
        LazyTexture::from_file(path, label)
    }

    pub fn create_descriptor_set_layout(
        &self,
        info: DescriptorSetLayoutDescriptor,
    ) -> DescriptorSetLayout {
        DescriptorSetLayout::create(&self.backend.device, info)
    }

    pub fn build_descriptor_set(&self, builder: &DescriptorSetBuilder) -> DescriptorSet {
        builder.build(&self.backend.device)
    }

    pub fn create_render_pipeline_layout(
        &self,
        descriptor_set_layouts: &[DescriptorSetLayout],
    ) -> PipelineLayout {
        PipelineLayout::create(&self.backend.device, descriptor_set_layouts)
    }

    pub fn create_shader_pair(&self, pair: ShaderPair) -> GraphicsShader {
        GraphicsShader::from_pair(&self.backend.device, pair)
    }

    pub fn create_render_pipeline(
        &self,
        pipeline_create_info: PipelineCreateInfo,
    ) -> RenderPipeline {
        RenderPipeline::create(&self.backend.device, pipeline_create_info)
    }

    // Convenience aliases for shorter method names
    pub fn create_pipeline_layout(&self, layouts: &[DescriptorSetLayout]) -> PipelineLayout {
        self.create_render_pipeline_layout(layouts)
    }

    pub fn create_pipeline(&self, create_info: PipelineCreateInfo) -> RenderPipeline {
        self.create_render_pipeline(create_info)
    }

    pub fn render<F>(&self, options: RenderOptions, execute: F) -> Result<()>
    where
        F: FnOnce(FrameBuilder),
    {
        self.backend.render(options, execute)
    }

    pub fn create_vertex_buffer_lazy(vertices: &[Vertex]) -> LazyBuffer<[Vertex]> {
        LazyBuffer::from_slice(vertices, BufferUsages::VERTEX, Some("Vertex Buffer"))
    }

    pub fn create_index_buffer_lazy(indicies: &[u32]) -> LazyBuffer<[u32]> {
        LazyBuffer::from_slice(indicies, BufferUsages::INDEX, Some("Index Buffer"))
    }

    pub fn create_unifrom_buffer_lazy<T: Pod>(uniform: &T) -> LazyBuffer<T> {
        LazyBuffer::new(
            uniform,
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            Some("Uniform Buffer"),
        )
    }
}
