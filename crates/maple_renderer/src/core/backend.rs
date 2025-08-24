use std::{iter, sync::Arc};

use anyhow::Result;
use bytemuck::Pod;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use wgpu::{
    BufferUsages, CommandEncoder, CommandEncoderDescriptor, Device, DeviceDescriptor, Instance,
    InstanceDescriptor, Queue, RenderPassDescriptor, RequestAdapterOptions, Surface,
    SurfaceConfiguration, SurfaceTexture, TextureFormat, TextureUsages, util::DeviceExt,
};

use crate::{
    core::{
        GraphicsShader, ShaderPair,
        buffer::Buffer,
        descriptor_set::{
            DescriptorBindingType, DescriptorSet, DescriptorSetDescriptor, DescriptorSetLayout,
            DescriptorSetLayoutDescriptor, DescriptorWrite, StageFlags,
        },
        frame_builder::FrameBuilder,
        pipeline::{PipelineCreateInfo, PipelineLayout, RenderPipeline},
    },
    types::Vertex,
};

#[derive(Debug)]
pub(crate) struct WGPUBackend {
    instance: Instance,
    pub device: Device,
    queue: Queue,
    size: [u32; 2],
    surface: Surface<'static>,
    pub surface_format: TextureFormat,
}

impl WGPUBackend {
    pub async fn init(
        window: Arc<impl HasDisplayHandle + HasWindowHandle + Send + Sync + 'static>,
        dimensions: [u32; 2],
    ) -> Result<Self> {
        let instance = Instance::new(&InstanceDescriptor::default());

        let adapter = instance
            .request_adapter(&RequestAdapterOptions::default())
            .await?;

        let (device, queue) = adapter.request_device(&DeviceDescriptor::default()).await?;

        let surface = instance.create_surface(window.clone())?;
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];

        let backend = Self {
            instance,
            device,
            queue,
            size: dimensions,
            surface,
            surface_format,
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
                width: self.size[0],
                height: self.size[1],
                desired_maximum_frame_latency: 2,
                present_mode: wgpu::PresentMode::AutoVsync,
            },
        );
    }

    pub fn resize(&mut self, new_size: [u32; 2]) {
        self.size = new_size;

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

    pub fn write_buffer<T: Pod>(&self, buffer: &Buffer<T>, value: &T) -> Result<()> {
        buffer.write(&self.queue, value)
    }

    pub fn create_descriptor_set_layout(
        &self,
        info: DescriptorSetLayoutDescriptor,
    ) -> DescriptorSetLayout {
        DescriptorSetLayout::create(&self.device, info)
    }

    pub fn create_descriptor_set<T>(&self, info: DescriptorSetDescriptor<T>) -> DescriptorSet {
        DescriptorSet::new(&self.device, info)
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

    pub fn render<F>(&self, pipeline: &RenderPipeline, execute: F) -> Result<()>
    where
        F: FnOnce(FrameBuilder),
    {
        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("render encoder"),
            });

        // scoped to drop render pass before we submit the encoder (render pass borrows encoder)
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
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

            render_pass.set_pipeline(&pipeline.backend);

            let frame_builder = FrameBuilder::new(render_pass);

            // the user defined commands
            execute(frame_builder);
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
