use anyhow::{Context, Result};
use bytemuck::Pod;

use std::sync::Arc;

use anyhow::anyhow;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::{
    core::{
        DescriptorSetBuilder, GraphicsShader, ShaderPair,
        backend::WGPUBackend,
        buffer::Buffer,
        descriptor_set::{DescriptorSet, DescriptorSetLayout, DescriptorSetLayoutDescriptor},
        frame_builder::FrameBuilder,
        pipeline::RenderPipeline,
        render_pass::{RenderPass, RenderPassWrapper},
    },
    types::Vertex,
};

/// The Renderer handles all rendering tasks for the engine as well as provides tools to help in
/// pass creation
pub struct Renderer {
    backend: RenderBackend,
    render_passes: Vec<RenderPassWrapper>,
}

/// what backend the renderer is using
#[derive(Debug)]
pub(crate) enum RenderBackend {
    /// wgpu backend
    Wgpu(WGPUBackend),
    /// headlss backend
    Headless,
}

impl Renderer {
    /// create a headless Renderer
    ///
    /// headless means that there is no gpu device or surface to render to and the renderer will
    /// panic if you try to
    pub fn headless() -> Self {
        Self {
            backend: RenderBackend::Headless,
            render_passes: Vec::new(),
        }
    }

    /// creates and initializes the renderer
    pub fn init<T>(window: Arc<T>, dimensions: [u32; 2]) -> Result<Self>
    where
        T: HasWindowHandle + HasDisplayHandle + Send + Sync + 'static,
    {
        let backend =
            RenderBackend::Wgpu(pollster::block_on(WGPUBackend::init(window, dimensions))?);

        Ok(Renderer {
            backend,
            render_passes: Vec::new(),
        })
    }

    /// resize the surface as well as render_passes that might need that
    pub fn resize(&mut self, dimensions: [u32; 2]) {
        self.render_passes
            .iter_mut()
            .for_each(|r| match r.pass.resize(dimensions) {
                Ok(_) => {}
                Err(e) => eprintln!("failed to resize render pass: {}, {}", r.context.name, e),
            });

        match &mut self.backend {
            RenderBackend::Wgpu(backend) => backend.resize(dimensions),
            _ => panic!("cant resize headless renderer"),
        }
    }

    /// create a vertex buffer
    pub fn create_vertex_buffer(&self, vertices: &[Vertex]) -> Buffer<[Vertex]> {
        match &self.backend {
            RenderBackend::Wgpu(backend) => backend.create_vertex_buffer(vertices),
            _ => panic!("could not create Vertex Buffer in headless mode"),
        }
    }

    /// create a index buffer
    pub fn create_index_buffer(&self, indicies: &[u32]) -> Buffer<[u32]> {
        match &self.backend {
            RenderBackend::Wgpu(backend) => backend.create_index_buffer(indicies),
            _ => panic!("could not create index Buffer in headless mode"),
        }
    }

    /// create a uniform buffer for use in descriptor sets
    pub fn create_uniform_buffer<T: Pod>(&self, data: &T) -> Buffer<T> {
        match &self.backend {
            RenderBackend::Wgpu(backend) => backend.create_uniform_buffer(data),
            _ => panic!("could not create uniform buffer in headless mode"),
        }
    }

    /// write to a buffer the buffer must implement COPY_DST so that its accessable
    pub fn write_buffer<T: Pod>(&self, buffer: &Buffer<T>, value: &T) -> Result<()> {
        match &self.backend {
            RenderBackend::Wgpu(backend) => backend.write_buffer(buffer, value),
            _ => Err(anyhow!("cannot write to a buffer in headless mode")),
        }
    }

    ///create the layout for a descriptor set
    pub fn create_descriptor_set_layout(
        &self,
        info: DescriptorSetLayoutDescriptor,
    ) -> DescriptorSetLayout {
        match &self.backend {
            RenderBackend::Wgpu(backend) => backend.create_descriptor_set_layout(info),
            _ => panic!("could not create descriptor set layout in headless mode"),
        }
    }

    /// build a descriptor set from a builder
    pub fn build_descriptor_set(&self, builder: &DescriptorSetBuilder) -> DescriptorSet {
        match &self.backend {
            RenderBackend::Wgpu(backend) => backend.build_descriptor_set(builder),
            _ => panic!("could not create descriptor set in headless mode"),
        }
    }

    /// create a shader from a pair (vertex and fragment)
    pub fn create_shader_pair(&self, pair: ShaderPair) -> GraphicsShader {
        match &self.backend {
            RenderBackend::Wgpu(backend) => backend.create_shader_pair(pair),
            _ => panic!("cant compile shader in headless mode"),
        }
    }

    /// add a node to the render graph
    pub fn add_render_node<T>(&mut self, mut node: T)
    where
        T: RenderPass + 'static,
    {
        // TODO implement non linear render graph
        let description = node.setup(self);

        let wrapper = match &mut self.backend {
            RenderBackend::Wgpu(backend) => {
                let color_format = backend.surface_format;
                RenderPassWrapper::create(
                    &backend.device,
                    Box::new(node),
                    color_format,
                    description,
                )
            }
            _ => panic!("could not add pass in headless mode"),
        };

        self.render_passes.push(wrapper);
    }

    /// begins the render passes within the render graph patent pending
    pub fn draw(&mut self) {
        let mut passes = std::mem::take(&mut self.render_passes);

        for pass in &mut passes {
            match pass.pass.draw(self, &pass.context.pipeline, &[]) {
                Ok(_) => {}
                Err(e) => eprintln!("failed to draw {}: {}", pass.context.name, e),
            };
        }

        self.render_passes = passes;
    }

    /// called within a pass and tells the renderer to render a defined command buffer made with
    /// FrameBuilder
    pub fn render<F>(&self, pipeline: &RenderPipeline, render_function: F) -> Result<()>
    where
        F: FnOnce(FrameBuilder),
    {
        match &self.backend {
            RenderBackend::Wgpu(backend) => backend
                .render(pipeline, render_function)
                .context("render call failed")?,
            _ => panic!("could not render while in headless mode"),
        };

        Ok(())
    }
}
