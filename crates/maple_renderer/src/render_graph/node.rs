//! this code is bad and needs to be improved

use maple_engine::Scene;

use crate::{
    core::{
        DepthCompare, DepthStencilOptions, RenderContext,
        descriptor_set::DescriptorSetLayout,
        pipeline::{PipelineCreateInfo, RenderPipeline},
        shader::GraphicsShader,
        texture::{Texture, TextureCreateInfo, TextureFormat, TextureUsage},
    },
    render_graph::graph::RenderGraphContext,
};

pub struct RenderNodeContext {
    /// shader to use
    shader: GraphicsShader,

    pipeline: RenderPipeline,

    target: RenderTarget,

    depth: Option<DepthStencilOptions>,
}

impl RenderNodeContext {
    pub fn shader(&self) -> &GraphicsShader {
        &self.shader
    }

    pub fn pipeline(&self) -> &RenderPipeline {
        &self.pipeline
    }

    pub fn target(&self) -> &RenderTarget {
        &self.target
    }

    pub fn depth_options(&self) -> Option<&DepthStencilOptions> {
        self.depth.as_ref()
    }

    pub fn update_depth_texture(&mut self, new_texture: Texture) {
        if let Some(depth_options) = &mut self.depth {
            depth_options.texture = new_texture;
        }
    }

    pub fn update_target(&mut self, new_target: RenderTarget) {
        self.target = new_target;

        if self.depth.is_some() {}
    }

    fn create_depth_texture(render_ctx: &RenderContext, target: &RenderTarget) -> Texture {
        let (width, height) = target.dimensions(render_ctx);

        render_ctx.create_texture(TextureCreateInfo {
            label: Some("depth texture"),
            width,
            height,
            format: crate::core::texture::TextureFormat::Depth32,
            usage: TextureUsage::RENDER_ATTACHMENT,
        })
    }

    fn recreate_depth_texture(&mut self, render_ctx: &RenderContext) {
        if let Some(depth_options) = &mut self.depth {
            let new_depth = Self::create_depth_texture(render_ctx, &self.target);
            depth_options.texture = new_depth;
        }
    }
}

#[derive(PartialEq, Eq)]
pub enum RenderTarget {
    Surface,
    Texture(Texture),
}

impl RenderTarget {
    /// gets the dimensions of the target  (width, height)
    pub fn dimensions(&self, render_ctx: &RenderContext) -> (u32, u32) {
        match self {
            RenderTarget::Surface => render_ctx.surface_size(),
            RenderTarget::Texture(tex) => (tex.width(), tex.height()),
        }
    }
}

pub struct RenderNodeDescriptor {
    pub shader: GraphicsShader,
    pub descriptor_set_layouts: Vec<DescriptorSetLayout>,
    pub target: RenderTarget,
    pub depth: Option<DepthCompare>,
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
        render_ctx: &RenderContext,
        pipeline_label: Option<&'static str>,
        pass: Box<dyn RenderNode>,
        color_format: TextureFormat,
        info: RenderNodeDescriptor,
    ) -> Self {
        let pipeline_layout = render_ctx.create_pipeline_layout(&info.descriptor_set_layouts);

        let depth = info.depth.as_ref().map(|compare_function| {
            let depth_texture = RenderNodeContext::create_depth_texture(render_ctx, &info.target);

            DepthStencilOptions {
                texture: depth_texture,
                compare: *compare_function,
                write_enabled: true,
            }
        });

        let pipeline = render_ctx.create_pipeline(PipelineCreateInfo {
            label: pipeline_label,
            shader: info.shader.clone(),
            layout: pipeline_layout,
            color_format,
            depth: depth.as_ref(),
        });

        let ctx = RenderNodeContext {
            shader: info.shader,
            pipeline,
            target: info.target,
            depth,
        };

        RenderNodeWrapper { context: ctx, pass }
    }

    pub fn resize(&mut self, render_ctx: &RenderContext, dimensions: [u32; 2]) {
        if matches!(self.context.target, RenderTarget::Surface) && self.context.depth.is_some() {
            self.context.recreate_depth_texture(render_ctx);
        }

        self.pass.resize(render_ctx, &mut self.context, dimensions);
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
