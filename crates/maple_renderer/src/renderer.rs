use anyhow::Result;
use std::{any::Any, sync::Arc};

use anyhow::anyhow;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use vulkano::buffer::BufferContents;

use crate::{buffer::Buffer, vulkan::VulkanBackend};

#[derive(Default)]
pub struct Renderer {
    backend: RenderBackend,
}

#[derive(Default, Debug)]
pub enum RenderBackend {
    Vulkan(VulkanBackend),
    #[default]
    Headless,
}

impl Renderer {
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

        Self { backend }
    }

    pub fn resize(&mut self, dimensions: [u32; 2]) {}

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
}
