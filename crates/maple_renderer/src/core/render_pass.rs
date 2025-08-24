use std::sync::Arc;

use anyhow::Result;
use wgpu::{Device, TextureFormat};

use crate::{
    core::{
        descriptor_set::DescriptorSetLayout,
        pipeline::{PipelineCreateInfo, PipelineLayout, RenderPipeline},
        renderer::Renderer,
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
    pub descriptor_set_layouts: Vec<DescriptorSetLayout>,
}

pub trait RenderPass {
    /// sets up the renderpass here is where you compile shaders, set up descritors, etc...
    fn setup(&mut self, renderer: &Renderer) -> RenderPassDescriptor;

    /// called every frame here is where you put logic to draw stuff
    fn draw(
        &mut self,
        renderer: &Renderer,
        pipeline: &RenderPipeline,
        drawables: &[&dyn Drawable],
    ) -> Result<()>;

    /// called when the window resizes if that is relavent to the pass
    fn resize(&mut self, dimensions: [u32; 2]) -> Result<()> {
        Ok(())
    }
}

pub(crate) struct RenderPassWrapper {
    pub context: RenderPassContext,
    pub pass: Box<dyn RenderPass>,
}

impl RenderPassWrapper {
    pub fn create(
        device: &Device,
        pass: Box<dyn RenderPass>,
        color_format: TextureFormat,
        info: RenderPassDescriptor,
    ) -> Self {
        let pipeline_layout = PipelineLayout::create(device, &info.descriptor_set_layouts);
        let pipeline = RenderPipeline::create(
            device,
            PipelineCreateInfo {
                label: Some(info.name),
                shader: info.shader.clone(),
                layout: pipeline_layout,
                color_format,
            },
        );

        let ctx = RenderPassContext {
            name: info.name,
            shader: info.shader,
            pipeline,
        };

        RenderPassWrapper { context: ctx, pass }
    }
}
