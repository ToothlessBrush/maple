use std::sync::Arc;

use vulkano::render_pass::RenderPass;

use crate::core::render_pass::BackendRenderpass;

pub struct VulkanRenderPass {
    pub render_pass: Arc<RenderPass>,
}

impl BackendRenderpass for VulkanRenderPass {}
