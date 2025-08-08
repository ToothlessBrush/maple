use std::sync::Arc;

use anyhow::Result;
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
    format::Format,
    pipeline::graphics::viewport::Viewport,
    render_pass::Framebuffer,
};

use crate::core::{
    pipeline::{Pipeline, RenderPipeline},
    renderer::Renderer,
    shader::GraphicsShader,
};

pub trait BackendRenderpass {}

pub struct RenderPassBackend {
    pub inner: Arc<dyn BackendRenderpass>,
}

pub struct RenderPassContext {
    /// name of the pass
    pub name: &'static str,
    /// shader to use
    pub shader: GraphicsShader,

    pub pipeline: RenderPipeline,
}

pub struct RenderPassDescriptor {
    pub name: &'static str,
    pub shader: Arc<GraphicsShader>,
    pub format: Format,
    pub depth_format: Option<Format>,
    pub viewport: Option<Viewport>,
}

pub trait RenderPass {
    /// sets up the renderpass here is where you compile shaders etc...
    fn setup(&self, renderer: &Renderer) -> &RenderPassDescriptor;
    /// called every frame here is where you put logic to draw stuff
    fn draw(
        &mut self,
        // TODO abstract the command buffer into api agnostic builder
        command_buffer_builder: &AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) -> Result<()>;
    /// called when the window resizes if that is relavent to the pass
    fn resize(&mut self, dimensions: [f32; 2]) -> Result<()> {
        Ok(())
    }
}

pub struct RenderPassWrapper {
    pub context: RenderPassContext,
    pub pass: Box<dyn RenderPass>,
}
