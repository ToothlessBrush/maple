use std::{any::Any, sync::Arc};

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::vulkan::VulkanBackend;

#[derive(Default)]
pub struct Renderer {
    backend: RenderBackend,
}

#[derive(Default)]
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
}
