use std::sync::Arc;

use anyhow::Result;
use vulkano::{render_pass::Framebuffer, shader::ShaderModule};

use crate::vulkan::shader::GraphicsShader;

pub struct RenderPassDescription {
    /// name of the pass
    pub name: &'static str,
    /// shader to use
    pub shader: GraphicsShader, // placeholder until I abstract shader
    /// if None will render to swapchain
    pub target: Option<Arc<Framebuffer>>, // placeholder until I abstract Framebuffer
}

pub trait RenderPass {
    fn description(&self) -> &RenderPassDescription;
    fn draw(&mut self) -> Result<()>;
    fn resize(&mut self, dimensions: [f32; 2]) -> Result<()>;
}
