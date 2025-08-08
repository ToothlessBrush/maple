use anyhow::Result;
use std::{any::Any, fs::read_to_string, path::Path, sync::Arc};

use anyhow::anyhow;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use vulkano::buffer::BufferContents;

use crate::{
    backend::vulkan::{VulkanBackend, shader::VulkanShader},
    core::{
        buffer::Buffer,
        render_pass::{RenderPass, RenderPassBackend, RenderPassContext, RenderPassWrapper},
        shader::GraphicsShader,
    },
};

#[derive(Default)]
pub struct Renderer {
    backend: RenderBackend,
    render_passes: Vec<RenderPassWrapper>,
}

#[derive(Default, Debug)]
pub enum RenderBackend {
    Vulkan(VulkanBackend),
    #[default]
    Headless,
}

impl Renderer {
    /// initialize the Renderer
    ///
    /// attempted to init vulkan before falling back to headless
    pub fn init(
        window: Arc<(impl HasDisplayHandle + HasWindowHandle + Send + Sync + Any)>,
        surface_dimensions: [u32; 2],
    ) -> Self {
        let backend = match VulkanBackend::init(window, surface_dimensions) {
            Ok(backend) => {
                println!("successfully created vulkan backend");
                RenderBackend::Vulkan(backend)
            }
            Err(e) => {
                eprintln!("failed to init vulkan: {e}");
                RenderBackend::Headless
            }
        };

        Self {
            backend,
            render_passes: vec![],
        }
    }

    /// resize the renderer when the window dimensions change
    pub fn resize(&mut self, dimensions: [u32; 2]) -> Result<()> {
        match &mut self.backend {
            RenderBackend::Vulkan(vulkan_backend) => vulkan_backend.resize(dimensions),
            _ => Ok(()),
        }
    }

    /// create a vertex buffer
    pub fn create_vertex_buffer<T, I>(&self, iter: I) -> Result<Buffer<T>>
    where
        T: BufferContents,
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        match &self.backend {
            RenderBackend::Vulkan(vulkan_backend) => {
                let vb = vulkan_backend.create_buffer_vertex(iter)?;
                Ok(vb.into())
            }
            _ => Err(anyhow!("could not create buffer with: {:?}", self.backend)),
        }
    }

    /// create index buffer
    pub fn create_index_buffer<T, I>(&self, iter: I) -> Result<Buffer<T>>
    where
        T: BufferContents,
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        match &self.backend {
            RenderBackend::Vulkan(vulkan_backend) => {
                let ib = vulkan_backend.create_buffer_index(iter)?;
                Ok(ib.into())
            }
            _ => Err(anyhow!("could not create buffer with: {:?}", self.backend)),
        }
    }

    /// create a uniform buffer
    pub fn create_uniform_buffer<T>(&self, data: T) -> Result<Buffer<T>>
    where
        T: BufferContents,
    {
        match &self.backend {
            RenderBackend::Vulkan(vulkan_backend) => {
                let ub = vulkan_backend.create_buffer_uniform(data)?;
                Ok(ub.into())
            }
            _ => Err(anyhow!("could not create buffer with: {:?}", self.backend)),
        }
    }

    pub fn create_renderpass<T>(&self) {
        match &self.backend {
            RenderBackend::Vulkan(vulkan_backend) => {}
            _ => {}
        }
    }

    pub fn create_shader(&self, vert: &Path, frag: &Path) -> Result<GraphicsShader> {
        let vert_source = read_to_string(vert)?;
        let frag_source = read_to_string(frag)?;

        match &self.backend {
            RenderBackend::Vulkan(vulkan_backend) => {
                let shader = VulkanShader::new(vulkan_backend, &vert_source, &frag_source)?;
                Ok(GraphicsShader {
                    inner: crate::core::shader::ShaderBackend::Vulkan(shader),
                })
            }
            _ => Err(anyhow!("could not create shader with: {:?}", self.backend)),
        }
    }

    pub fn draw(&mut self) -> Result<()> {
        match &self.backend {
            RenderBackend::Vulkan(vulkan_backend) => {}
            _ => Ok(()),
        }
    }

    /// add a pass
    pub fn add_pass<T>(&mut self, pass: T) -> Result<()>
    where
        T: RenderPass + 'static,
    {
        let info = pass.setup(&self);

        match &self.backend {
            RenderBackend::Vulkan(vulkan_backend) => {
                let renderpass = vulkan_backend.create_render_pass(info)?;
                let renderpass = Arc::new(RenderPassBackend {
                    inner: Arc::new(renderpass),
                });

                let context = RenderPassContext {
                    name: info.name,
                    shader: info.shader.clone(),
                    render_pass: renderpass,
                };

                let wrapper = RenderPassWrapper {
                    context,
                    pass: Box::new(pass),
                };

                let pipeline = vulkan_backend.create_pipeline(
                    info.shader.try_into(),
                    renderpass,
                    vulkan_backend.viewport,
                );

                self.render_passes.push(wrapper);
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
