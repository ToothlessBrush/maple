use anyhow::{Context, Result};
use bytemuck::Pod;
use wgpu::TextureFormat;

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
        texture::{Sampler, SamplerOptions, Texture, TextureCreateInfo},
    },
    render_graph::{
        graph::{GraphBuilder, RenderGraph},
        node::{RenderNode, RenderNodeContext, RenderNodeWrapper, RenderTarget},
    },
    types::Vertex,
};

// TODO create a render context to avoid passing itself to the graph

/// The Renderer handles all rendering tasks for the engine as well as provides tools to help in
/// pass creation
pub struct Renderer {
    pub context: RenderContext,
    pub render_graph: RenderGraph,
}

pub struct RenderContext {
    backend: RenderBackend,
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
            context: RenderContext {
                backend: RenderBackend::Headless,
            },
            render_graph: RenderGraph::default(),
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
            context: RenderContext { backend },
            render_graph: RenderGraph::default(),
        })
    }

    /// resize the surface as well as render_passes that might need that
    pub fn resize(&mut self, dimensions: [u32; 2]) {
        match self.render_graph.resize(dimensions) {
            Ok(()) => {}
            Err(e) => {
                eprintln!("failed to resize render graph: {e}")
            }
        };

        match &mut self.context.backend {
            RenderBackend::Wgpu(backend) => backend.resize(dimensions),
            _ => panic!("cant resize headless renderer"),
        }
    }

    pub fn graph<'a>(&'a mut self) -> GraphBuilder<'a> {
        GraphBuilder::create(self)
    }

    /// begins the render passes within the render graph patent pending
    pub fn begin_draw(&mut self) -> Result<()> {
        self.render_graph.render(&self.context)?;

        Ok(())
    }
    /// add a node to the render graph
    pub(crate) fn setup_render_node<T>(
        &mut self,
        label: Option<&'static str>,
        mut node: T,
    ) -> RenderNodeWrapper
    where
        T: RenderNode + 'static,
    {
        // TODO implement non linear render graph
        let description = node.setup(&self.context, &mut self.render_graph.context);

        match &self.context.backend {
            RenderBackend::Wgpu(backend) => {
                let color_format: TextureFormat =
                    if let RenderTarget::Texture(texture) = &description.target {
                        texture.format().into()
                    } else {
                        backend.surface_format
                    };

                RenderNodeWrapper::create(
                    &backend.device,
                    label,
                    Box::new(node),
                    color_format,
                    description,
                )
            }
            _ => panic!("could not add pass in headless mode"),
        }
    }
}

impl RenderContext {
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

    pub fn create_storage_buffer<T: Pod>(&self, data: &T) -> Buffer<T> {
        match &self.backend {
            RenderBackend::Wgpu(backend) => backend.create_storage_buffer(data),
            _ => panic!("could not create storage buffer in headless mode"),
        }
    }

    pub fn create_storage_buffer_slice<T: Pod>(&self, data: &[T]) -> Buffer<[T]> {
        match &self.backend {
            RenderBackend::Wgpu(backend) => backend.create_storage_buffer_from_slice(data),
            _ => panic!("could not create storage buffer in headless mode"),
        }
    }

    pub fn create_sized_storage_buffer<T: Pod>(&self, len: usize) -> Buffer<[T]> {
        match &self.backend {
            RenderBackend::Wgpu(backend) => backend.create_sized_storage_buffer(len),
            _ => panic!("could not create storage buffer in headless mode"),
        }
    }

    /// write to a buffer the buffer must implement COPY_DST so that its accessable
    pub fn write_buffer<T: Pod>(&self, buffer: &Buffer<T>, value: &T) -> Result<()> {
        match &self.backend {
            RenderBackend::Wgpu(backend) => backend.write_buffer(buffer, value),
            _ => Err(anyhow!("cannot write to a buffer in headless mode")),
        }
    }

    pub fn create_texture(&self, info: TextureCreateInfo) -> Texture {
        match &self.backend {
            RenderBackend::Wgpu(backend) => backend.create_texture(info),
            _ => panic!("could not create texture in headless mode"),
        }
    }

    pub fn create_sampler(&self, options: SamplerOptions) -> Sampler {
        match &self.backend {
            RenderBackend::Wgpu(backend) => backend.create_sampler(options),
            _ => panic!("could not create sampler in headless mode"),
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

    /// called within a pass and tells the renderer to render a defined command buffer made with
    /// FrameBuilder
    pub fn render<F>(&self, ctx: &RenderNodeContext, render_function: F) -> Result<()>
    where
        F: FnOnce(FrameBuilder),
    {
        match &self.backend {
            RenderBackend::Wgpu(backend) => backend
                .render(ctx, render_function)
                .context("render call failed")?,
            _ => panic!("could not render while in headless mode"),
        };

        Ok(())
    }
}
