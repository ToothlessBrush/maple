use anyhow::Result;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

pub struct WindowSize<T> {
    x: T,
    y: T,
}

pub struct VulkanBackend {}

impl VulkanBackend {
    pub fn init<T>(
        window: &(impl HasDisplayHandle + HasWindowHandle),
        dimensions: WindowSize<T>,
    ) -> Result<Self> {
        Ok(Self {})
    }
}
