use std::sync::Arc;

use anyhow::Result;
use wgpu::{Device, TextureFormat};

use crate::{
    core::{
        descriptor_set::DescriptorSetLayout,
        pipeline::{PipelineCreateInfo, PipelineLayout, RenderPipeline},
        renderer::Renderer,
        shader::GraphicsShader,
        texture::Texture,
    },
    render_graph::graph::RenderGraphContext,
    types::{
        drawable::{self, Drawable},
        world::World,
    },
};

pub struct RenderNodeContext {
    /// shader to use
    pub shader: GraphicsShader,

    pub pipeline: RenderPipeline,

    pub(crate) target: RenderTarget,
}

#[derive(PartialEq, Eq)]
pub enum RenderTarget {
    Surface,
    Texture(Texture),
}

pub struct RenderNodeDescriptor {
    pub shader: GraphicsShader,
    pub descriptor_set_layouts: Vec<DescriptorSetLayout>,
    pub target: RenderTarget,
}

pub trait RenderNode {
    /// sets up the renderpass here is where you compile shaders, set up descritors, etc...
    fn setup(&mut self, renderer: &Renderer) -> RenderNodeDescriptor;

    /// called every frame here is where you put logic to draw stuff
    fn draw<'a>(
        &mut self,
        renderer: &Renderer,
        node_ctx: &mut RenderNodeContext,
        graph_ctx: &mut RenderGraphContext,
        world: World<'a>,
    ) -> Result<()>;

    /// called when the window resizes if that is relavent to the pass
    fn resize(&mut self, _dimensions: [u32; 2]) -> Result<()> {
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
        pipeline_label: Option<&'static str>,
        pass: Box<dyn RenderNode>,
        color_format: TextureFormat,
        info: RenderNodeDescriptor,
    ) -> Self {
        let pipeline_layout = PipelineLayout::create(device, &info.descriptor_set_layouts);
        let pipeline = RenderPipeline::create(
            device,
            PipelineCreateInfo {
                label: pipeline_label,
                shader: info.shader.clone(),
                layout: pipeline_layout,
                color_format,
            },
        );

        let ctx = RenderNodeContext {
            shader: info.shader,
            pipeline,
            target: info.target,
        };

        RenderNodeWrapper { context: ctx, pass }
    }
}
