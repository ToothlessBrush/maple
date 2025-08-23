use std::sync::Arc;

use anyhow::Result;

use crate::{
    core::{
        command_buffer_builder::CommandBufferBuilder, pipeline::RenderPipeline, renderer::Renderer,
        shader::GraphicsShader,
    },
    types::drawable::{self, Drawable},
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
    pub shader: GraphicsShader,
}

pub trait RenderPass {
    /// sets up the renderpass here is where you compile shaders, set up descritors, etc...
    fn setup(&mut self, renderer: &Renderer) -> RenderPassDescriptor;

    /// called every frame here is where you put logic to draw stuff
    fn draw(&mut self, renderer: &Renderer, drawables: &[&dyn Drawable]) -> Result<()>;

    /// called when the window resizes if that is relavent to the pass
    fn resize(&mut self, dimensions: [f32; 2]) -> Result<()> {
        Ok(())
    }
}

pub struct RenderPassWrapper {
    pub context: RenderPassContext,
    pub pass: Box<dyn RenderPass>,
}
