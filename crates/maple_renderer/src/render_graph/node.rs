use maple_engine::Scene;
use wgpu::{Device, TextureFormat};

use crate::{
    core::{
        RenderContext,
        descriptor_set::DescriptorSetLayout,
        pipeline::{PipelineCreateInfo, PipelineLayout, RenderPipeline},
        shader::GraphicsShader,
        texture::{DepthStencilOptions, Texture},
    },
    render_graph::graph::RenderGraphContext,
};

pub struct RenderNodeContext {
    /// shader to use
    pub shader: GraphicsShader,

    pub pipeline: RenderPipeline,

    pub(crate) target: RenderTarget,

    pub(crate) depth: Option<DepthStencilOptions>,
}

impl RenderNodeContext {
    pub fn update_depth_texture(&mut self, new_texture: Texture) {
        if let Some(depth_options) = &mut self.depth {
            depth_options.texture = new_texture;
        }
    }
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
    pub depth: Option<DepthStencilOptions>,
}

pub trait RenderNode {
    /// sets up the renderpass here is where you compile shaders, set up descritors, etc...
    fn setup(
        &mut self,
        render_ctx: &RenderContext,
        graph_ctx: &mut RenderGraphContext,
    ) -> RenderNodeDescriptor;

    /// called every frame here is where you put logic to draw stuff
    fn draw(
        &mut self,
        renderer_ctx: &RenderContext,
        node_ctx: &mut RenderNodeContext,
        graph_ctx: &mut RenderGraphContext,
        scene: &Scene,
    );

    /// called when the window resizes if that is relavent to the pass
    #[allow(unused)]
    fn resize(
        &mut self,
        render_ctx: &RenderContext,
        node_ctx: &mut RenderNodeContext,
        dimensions: [u32; 2],
    ) {
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
                depth: info.depth.as_ref(),
            },
        );

        let ctx = RenderNodeContext {
            shader: info.shader,
            pipeline,
            target: info.target,
            depth: info.depth,
        };

        RenderNodeWrapper { context: ctx, pass }
    }
}

pub struct Marker;

impl RenderNode for Marker {
    fn setup(
        &mut self,
        render_ctx: &RenderContext,
        _graph_ctx: &mut RenderGraphContext,
    ) -> RenderNodeDescriptor {
        // shaders are required to have a shader attached so create a simple dummy shader
        let dummy_shader = render_ctx.create_shader_pair(crate::core::ShaderPair::Glsl {
            vert: r#"#version 450
void main() {
    gl_Position = vec4(0.0);
}"#,
            frag: r#"#version 450
layout(location = 0) out vec4 outColor;
void main() {
    outColor = vec4(0.0, 0.0, 0.0, 0.0);
}"#,
        });

        RenderNodeDescriptor {
            shader: dummy_shader,
            descriptor_set_layouts: vec![],
            target: RenderTarget::Surface,
            depth: None,
        }
    }

    fn draw(
        &mut self,
        _renderer_ctx: &RenderContext,
        _node_ctx: &mut RenderNodeContext,
        _graph_ctx: &mut RenderGraphContext,
        _scene: &Scene,
    ) {
        // nop
    }
}
