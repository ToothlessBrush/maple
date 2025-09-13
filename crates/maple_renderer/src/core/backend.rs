use std::{iter, sync::Arc};

use anyhow::Result;
use bytemuck::Pod;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use wgpu::{
    BufferUsages, CommandEncoder, CommandEncoderDescriptor, Device, DeviceDescriptor, Instance,
    InstanceDescriptor, PresentMode, Queue, RenderPassDescriptor, RequestAdapterOptions, Surface,
    SurfaceConfiguration, SurfaceTexture, TextureFormat, TextureUsages, TextureView,
    util::DeviceExt,
};

use crate::{
    core::{
        DescriptorSetBuilder, GraphicsShader, ShaderPair,
        buffer::Buffer,
        descriptor_set::{
            DescriptorBindingType, DescriptorSet, DescriptorSetDescriptor, DescriptorSetLayout,
            DescriptorSetLayoutDescriptor, DescriptorWrite, StageFlags,
        },
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

#[derive(Debug)]
pub(crate) struct WGPUBackend {
    _instance: Instance,
    pub device: Device,
    queue: Queue,
    surface: Surface<'static>,
    pub surface_format: TextureFormat,
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
        let surface_format = cap.formats[0];

        let backend = Self {
            _instance: instance,
            device,
            queue,
            surface,
            surface_format,
            config,
        };

        backend.configure_surface();

        Ok(backend)
    }

    fn configure_surface(&self) {
        self.surface.configure(
            &self.device,
            &SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: self.surface_format,
                view_formats: vec![self.surface_format.add_srgb_suffix()],
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

    pub fn write_buffer<T: Pod>(&self, buffer: &Buffer<T>, value: &T) -> Result<()> {
        buffer.write(&self.queue, value)
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
            surface_tex: Option<wgpu::SurfaceTexture>, // keep alive until after submit
        }

        let prepared = match &ctx.target {
            RenderTarget::Surface => {
                let surface_tex = self.surface.get_current_texture()?;
                let view = surface_tex
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                PreparedTarget {
                    view,
                    surface_tex: Some(surface_tex),
                }
            }
            RenderTarget::Texture(t) => PreparedTarget {
                view: t.create_view().inner,
                surface_tex: None,
            },
        };

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &prepared.view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&ctx.pipeline.backend);

            let frame_builder = FrameBuilder::new(render_pass);
            execute(frame_builder);
        }

        self.queue.submit(std::iter::once(encoder.finish()));

        // Present only if we actually drew to the surface
        if let Some(surface_tex) = prepared.surface_tex {
            surface_tex.present();
        }

        Ok(())
    }
}
