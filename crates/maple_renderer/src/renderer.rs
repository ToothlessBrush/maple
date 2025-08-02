use anyhow::Result;
use std::{any::Any, sync::Arc};

use anyhow::anyhow;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use vulkano::buffer::BufferContents;

use crate::{buffer::Buffer, render_pass::RenderPass, vulkan::VulkanBackend};

#[derive(Default)]
pub struct Renderer {
    backend: RenderBackend,
    render_passes: Vec<Box<dyn RenderPass>>,
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
    pub fn resize(&mut self, dimensions: [u32; 2]) {
        match &mut self.backend {
            RenderBackend::Vulkan(vulkan_backend) => {
                vulkan_backend.resize(dimensions);
            }
            _ => {}
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

    pub fn draw(&mut self) -> Result<()> {
        Ok(())
    }

    /// add a pass
    pub fn add_pass<T>(&mut self, pass: T)
    where
        T: RenderPass + 'static,
    {
        self.render_passes.push(Box::new(pass));
    }
}
