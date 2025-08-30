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

pub struct RenderNodeContext {
    /// name of the pass
    pub name: &'static str,
    /// shader to use
    pub shader: GraphicsShader,

    pub pipeline: RenderPipeline,
}

pub struct RenderNodeDescriptor {
    pub name: &'static str,
    pub shader: GraphicsShader,
    pub descriptor_set_layouts: Vec<DescriptorSetLayout>,
}

pub trait RenderNode {
    /// sets up the renderpass here is where you compile shaders, set up descritors, etc...
    fn setup(&mut self, renderer: &Renderer) -> RenderNodeDescriptor;

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

pub(crate) struct RenderNodeWrapper {
    pub context: RenderNodeContext,
    pub pass: Box<dyn RenderNode>,
}

impl RenderNodeWrapper {
    pub fn create(
        device: &Device,
        pass: Box<dyn RenderNode>,
        color_format: TextureFormat,
        info: RenderNodeDescriptor,
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

        let ctx = RenderNodeContext {
            name: info.name,
            shader: info.shader,
            pipeline,
        };

        RenderNodeWrapper { context: ctx, pass }
    }
}
