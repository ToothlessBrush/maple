use std::sync::Arc;

use anyhow::anyhow;
use vulkano::{
    pipeline::{self, GraphicsPipeline, Pipeline},
    render_pass::RenderPass,
};

use crate::core::pipeline::{PipelineBackend, RenderPipeline};

pub struct VulkanPass {
    pipeline: Arc<RenderPass>,
}

#[derive(Clone)]
pub struct VulkanPipeline {
    inner: Arc<GraphicsPipeline>,
}

impl VulkanPipeline {
    pub fn unbox(&self) -> Arc<GraphicsPipeline> {
        self.inner.clone()
    }
}

impl From<RenderPipeline> for VulkanPipeline {
    fn from(value: RenderPipeline) -> Self {
        if let PipelineBackend::VK(p) = value.backend {
            p
        } else {
            unreachable!(
                "RenderPipeline::From expected Vulkan backend for pipeline but got something else"
            )
        }
    }
}
