use anyhow::Result;
use bytemuck::Pod;

use std::{any::Any, fs::read_to_string, path::Path, sync::Arc};

use anyhow::anyhow;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::{
    core::{
        GraphicsShader, ShaderPair,
        backend::WGPUBackend,
        buffer::Buffer,
        descriptor_set::{
            DescriptorSet, DescriptorSetDescriptor, DescriptorSetLayout,
            DescriptorSetLayoutDescriptor,
        },
        frame_builder::FrameBuilder,
        pipeline::RenderPipeline,
        render_pass::{RenderPass, RenderPassWrapper},
    },
    types::Vertex,
};

#[derive(Default)]
pub struct Renderer {
    backend: RenderBackend,
    render_passes: Vec<RenderPassWrapper>,
}

#[derive(Default, Debug)]
pub enum RenderBackend {
    WGPU(WGPUBackend),
    #[default]
    Headless,
}

impl Renderer {
    pub fn init(
        window: Arc<impl HasDisplayHandle + HasWindowHandle + Send + Sync + 'static>,
        dimensions: [u32; 2],
    ) -> Result<Self> {
        let backend =
            RenderBackend::WGPU(pollster::block_on(WGPUBackend::init(window, dimensions))?);

        Ok(Renderer {
            backend,
            render_passes: Vec::new(),
        })
    }

    pub fn resize(&mut self, dimensions: [u32; 2]) {
        self.render_passes
            .iter_mut()
            .for_each(|r| match r.pass.resize(dimensions) {
                Ok(_) => {}
                Err(e) => eprint!("failed to resize render pass: {}, {}", r.context.name, e),
            });

        match &mut self.backend {
            RenderBackend::WGPU(backend) => backend.resize(dimensions),
            _ => panic!("cant resize headless renderer"),
        }
    }

    pub fn create_vertex_buffer(&self, vertices: &[Vertex]) -> Buffer<[Vertex]> {
        match &self.backend {
            RenderBackend::WGPU(backend) => backend.create_vertex_buffer(vertices),
            _ => panic!("could not create Vertex Buffer in headless mode"),
        }
    }

    pub fn create_index_buffer(&self, indicies: &[u32]) -> Buffer<[u32]> {
        match &self.backend {
            RenderBackend::WGPU(backend) => backend.create_index_buffer(indicies),
            _ => panic!("could not create index Buffer in headless mode"),
        }
    }

    pub fn create_uniform_buffer<T: Pod>(&self, data: &T) -> Buffer<T> {
        match &self.backend {
            RenderBackend::WGPU(backend) => backend.create_uniform_buffer(data),
            _ => panic!("could not create uniform buffer in headless mode"),
        }
    }

    pub fn write_buffer<T: Pod>(&self, buffer: &Buffer<T>, value: &T) -> Result<()> {
        match &self.backend {
            RenderBackend::WGPU(backend) => backend.write_buffer(buffer, value),
            _ => Err(anyhow!("cannot write to a buffer in headless mode")),
        }
    }

    pub fn create_descriptor_set_layout(
        &self,
        info: DescriptorSetLayoutDescriptor,
    ) -> DescriptorSetLayout {
        match &self.backend {
            RenderBackend::WGPU(backend) => backend.create_descriptor_set_layout(info),
            _ => panic!("could not create descriptor set layout in headless mode"),
        }
    }

    pub fn create_descriptor_set<T>(&self, info: DescriptorSetDescriptor<T>) -> DescriptorSet {
        match &self.backend {
            RenderBackend::WGPU(backend) => backend.create_descriptor_set(info),
            _ => panic!("could not create descriptor set in headless mode"),
        }
    }

    pub fn create_shader_pair(&self, pair: ShaderPair) -> GraphicsShader {
        match &self.backend {
            RenderBackend::WGPU(backend) => backend.create_shader_pair(pair),
            _ => panic!("cant compile shader in headless mode"),
        }
    }

    pub fn add_pass<T>(&mut self, mut pass: T)
    where
        T: RenderPass + 'static,
    {
        let description = pass.setup(self);

        let wrapper = match &mut self.backend {
            RenderBackend::WGPU(backend) => {
                let color_format = backend.surface_format;
                RenderPassWrapper::create(
                    &backend.device,
                    Box::new(pass),
                    color_format,
                    description,
                )
            }
            _ => panic!("could not add pass in headless mode"),
        };

        self.render_passes.push(wrapper);
    }

    pub fn draw(&mut self) {
        let mut passes = std::mem::take(&mut self.render_passes);

        for pass in &mut passes {
            pass.pass.draw(&self, &pass.context.pipeline, &[]);
        }

        self.render_passes = passes;
    }

    pub fn render<F>(&self, pipeline: &RenderPipeline, render_function: F)
    where
        F: FnOnce(FrameBuilder),
    {
        match &self.backend {
            RenderBackend::WGPU(backend) => backend.render(pipeline, render_function),
            _ => panic!("could not render while in headless mode"),
        };
    }
}
