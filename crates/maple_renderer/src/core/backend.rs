use std::{cell::RefCell, error::Error, rc::Rc, sync::Arc};

use anyhow::Result;
use bytemuck::Pod;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use wgpu::{
    BufferUsages, Device, DeviceDescriptor, Instance, InstanceDescriptor, Operations, PresentMode,
    Queue, RenderPassDepthStencilAttachment, RequestAdapterOptions, Surface, SurfaceConfiguration,
    SurfaceTexture, TextureFormat, TextureUsages, wgc::command::RenderPassColorAttachment,
};

use crate::{
    core::{
        DescriptorSetBuilder, GraphicsShader, ShaderPair,
        buffer::Buffer,
        descriptor_set::{DescriptorSet, DescriptorSetLayout, DescriptorSetLayoutDescriptor},
        frame_builder::FrameBuilder,
        pipeline::{PipelineCreateInfo, PipelineLayout, RenderPipeline},
        texture::{Sampler, SamplerOptions, Texture, TextureCreateInfo},
    },
    render_graph::node::{RenderNodeContext, RenderTarget},
    types::{
        Vertex,
        render_config::{RenderConfig, VsyncMode},
    },
};

use super::{LazyBufferable, texture};

#[derive(Debug)]
pub(crate) struct WGPUBackend {
    _instance: Instance,
    pub device: Device,
    queue: Queue,
    surface: Surface<'static>,
    current_surface_texture: Option<SurfaceTexture>,
    pub surface_format: texture::TextureFormat,
    config: RenderConfig,
}

impl WGPUBackend {
    pub async fn init<T>(window: Arc<T>, config: RenderConfig) -> Result<Self>
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

        let backend = Self {
            _instance: instance,
            device,
            queue,
            surface,
            current_surface_texture: None,
            surface_format,
            config,
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

        self.configure_surface();
    }

    pub fn change_vsync(&mut self, mode: VsyncMode) {
        self.config.vsync = mode;

        self.configure_surface();
    }

    pub fn create_vertex_buffer(&self, vertices: &[Vertex]) -> Buffer<[Vertex]> {
        Buffer::from_slice(
            &self.device,
            vertices,
            BufferUsages::VERTEX,
            "Vertex Buffer",
        )
    }

    pub fn create_index_buffer(&self, indices: &[u32]) -> Buffer<[u32]> {
        Buffer::from_slice(&self.device, indices, BufferUsages::INDEX, "Index Buffer")
    }

    pub fn create_uniform_buffer<T: Pod>(&self, uniform: &T) -> Buffer<T> {
        Buffer::from(
            &self.device,
            uniform,
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            "Uniform Buffer",
        )
    }

    pub fn create_storage_buffer<T: Pod>(&self, data: &T) -> Buffer<T> {
        Buffer::from(
            &self.device,
            data,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            "Storage Buffer",
        )
    }

    pub fn create_empty_storage_buffer<T: Pod>(&self) -> Buffer<T> {
        Buffer::empty(
            &self.device,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            "storage buffer",
        )
    }

    pub fn create_storage_buffer_from_slice<T: Pod>(&self, data: &[T]) -> Buffer<[T]> {
        Buffer::from_slice(
            &self.device,
            data,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            "Storage Buffer",
        )
    }

    pub fn create_sized_storage_buffer<T: Pod>(&self, len: usize) -> Buffer<[T]> {
        Buffer::from_size(
            &self.device,
            len,
            BufferUsages::STORAGE | BufferUsages::COPY_DST,
            "storage buffer",
        )
    }

    /// syncs the lazy buffer with the gpu buffer
    ///
    /// call this when you need to sync that data but dont need to read the buffer because its
    /// already in a descriptor set or other gpu buffer
    pub fn sync_lazy_buffer<T, B>(&self, lazy_buffer: &B)
    where
        B: LazyBufferable<T>,
        T: ?Sized,
    {
        lazy_buffer.sync(&self.queue)
    }

    /// materializes the lazy buffer into a buffer both represent the same gpu buffer
    ///
    /// keep in mind when you write to the lazy buffer it wont update the gpu buffer until you read
    /// or sync the buffer again
    pub fn get_buffer<T, B>(&self, lazy_buffer: &B) -> Buffer<T>
    where
        B: LazyBufferable<T>,
        T: ?Sized,
    {
        lazy_buffer.get_buffer(&self.device, &self.queue)
    }

    pub fn write_buffer<T: Pod>(&self, buffer: &Buffer<T>, value: &T) {
        buffer.write(&self.queue, value)
    }

    pub fn write_buffer_slice<T: Pod>(&self, buffer: &Buffer<[T]>, data: &[T]) {
        buffer.write(&self.queue, data)
    }

    pub fn create_texture(&self, info: TextureCreateInfo) -> Texture {
        Texture::create(&self.device, info)
    }

    pub fn create_sampler(&self, options: SamplerOptions) -> Sampler {
        Texture::create_sampler(&self.device, options)
    }

    pub fn create_descriptor_set_layout(
        &self,
        info: DescriptorSetLayoutDescriptor,
    ) -> DescriptorSetLayout {
        DescriptorSetLayout::create(&self.device, info)
    }

    pub fn build_descriptor_set(&self, builder: &DescriptorSetBuilder) -> DescriptorSet {
        builder.build(&self.device)
    }

    pub fn create_render_pipeline_layout(
        &self,
        descriptor_set_layouts: &[DescriptorSetLayout],
    ) -> PipelineLayout {
        PipelineLayout::create(&self.device, descriptor_set_layouts)
    }

    pub fn create_shader_pair(&self, pair: ShaderPair) -> GraphicsShader {
        GraphicsShader::from_pair(&self.device, pair)
    }

    pub fn create_render_pipeline(
        &self,
        pipeline_create_info: PipelineCreateInfo,
    ) -> RenderPipeline {
        RenderPipeline::create(&self.device, pipeline_create_info)
    }

    pub fn render<F>(&self, ctx: &RenderNodeContext, execute: F) -> Result<()>
    where
        F: FnOnce(FrameBuilder),
    {
        // Prepare the render target only as needed
        struct PreparedTarget {
            view: wgpu::TextureView,
        }

        let mut prepared = Vec::new();

        for target in ctx.target() {
            match target {
                RenderTarget::Surface => {
                    let surface_tex = self.get_surface_texture().unwrap();
                    let view = surface_tex
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default());
                    prepared.push(PreparedTarget { view });
                }
                RenderTarget::Texture(t) => {
                    prepared.push(PreparedTarget {
                        view: t.create_view().inner,
                    });
                }
            }
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        {
            let depth_view = ctx
                .depth_options()
                .map_to_option()
                .map(|view| view.texture.create_view());

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
                .map(|prepared| {
                    Some(wgpu::RenderPassColorAttachment {
                        view: &prepared.view,
                        resolve_target: None,
                        depth_slice: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.1,
                                b: 0.1,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })
                })
                .collect();

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &color_attachments,
                depth_stencil_attachment,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&ctx.pipeline().backend);

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
