use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::vulkan::VulkanBackend;

pub struct Renderer {
    backend: RenderBackend,
}

pub enum RenderBackend {
    Vulkan(VulkanBackend),
    Headless,
}

impl Renderer {
    fn init(window: &impl HasDisplayHandle) -> Self {
        let backend = match VulkanBackend::init() {
            Ok(backend) => RenderBackend::Vulkan(backend),
            Err(e) => {
                eprintln!("failed to init vulkan: {e}");
                RenderBackend::Headless
            }
        };

        Self { backend }
    }
}
