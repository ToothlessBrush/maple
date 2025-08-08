use std::sync::Arc;

use crate::backend::vulkan::pipeline::{self, VulkanPipeline};

pub trait Pipeline {}

#[derive(Clone)]
pub enum PipelineBackend {
    VK(pipeline::VulkanPipeline),
}

#[derive(Clone)]
pub struct RenderPipeline {
    pub backend: PipelineBackend,
}

impl From<VulkanPipeline> for RenderPipeline {
    fn from(value: VulkanPipeline) -> Self {
        Self {
            backend: PipelineBackend::VK(value),
        }
    }
}
