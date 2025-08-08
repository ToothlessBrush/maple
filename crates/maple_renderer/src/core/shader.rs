use std::sync::Arc;

use anyhow::{Result, anyhow};

use crate::backend::vulkan::shader::VulkanShader;

pub enum ShaderBackend {
    Vulkan(VulkanShader),
}

pub struct GraphicsShader {
    pub inner: ShaderBackend,
}

impl From<VulkanShader> for GraphicsShader {
    fn from(value: VulkanShader) -> Self {
        Self {
            inner: ShaderBackend::Vulkan(value),
        }
    }
}
