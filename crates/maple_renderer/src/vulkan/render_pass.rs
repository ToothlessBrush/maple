use std::sync::Arc;

use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer};
use vulkano::pipeline::GraphicsPipeline;
use vulkano::render_pass::{Framebuffer, RenderPass as VkRenderPass};

use crate::vulkan::shader::GraphicsShader;

pub struct VulkanRenderPass {
    pub pipeline: Arc<GraphicsPipeline>,
    pub render_pass: Arc<VkRenderPass>,
    pub framebuffers: Vec<Arc<Framebuffer>>,
    pub shader: GraphicsShader,
}
