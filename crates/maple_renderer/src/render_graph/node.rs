use maple_engine::Scene;

use crate::{
    core::{
        CullMode, DepthCompare, DepthStencilOptions, RenderContext,
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
    /// pipeline to use (None for resource-only nodes that don't render)
    pipeline: Option<RenderPipeline>,
    target: Vec<RenderTarget>,
    depth: DepthMode,
}

impl RenderNodeContext {
    pub fn shader(&self) -> &GraphicsShader {
        &self.shader
    }

    pub fn pipeline(&self) -> Option<&RenderPipeline> {
        self.pipeline.as_ref()
    }

    pub fn target(&self) -> &Vec<RenderTarget> {
        &self.target
    }

    pub fn targets(&self) -> &[RenderTarget] {
        &self.target
    }

    pub fn depth_options(&self) -> &DepthMode {
        &self.depth
    }

    pub fn update_depth_texture(&mut self, new_texture: Texture) {
        if let DepthMode::Auto(depth_options) = &mut self.depth {
            println!("updating depth texture even though its automatically managed");
            depth_options.texture = new_texture;
        } else if let DepthMode::Manual(depth_options) = &mut self.depth {
            depth_options.texture = new_texture
        }
    }

    pub fn update_target(&mut self, render_ctx: &RenderContext, new_targets: Vec<RenderTarget>) {
        self.target = new_targets;
        if let DepthMode::Auto(depth_options) = &mut self.depth {
            let depth_tex = Self::create_depth_texture(render_ctx, &self.target);
            depth_options.texture = depth_tex;
        }
    }

    pub fn add_target(&mut self, render_ctx: &RenderContext, new_target: RenderTarget) {
        self.target.push(new_target);
        // Recreate depth texture if auto-managed and this affects sizing
        if let DepthMode::Auto(depth_options) = &mut self.depth {
            let depth_tex = Self::create_depth_texture(render_ctx, &self.target);
            depth_options.texture = depth_tex;
        }
    }

    fn create_depth_texture(render_ctx: &RenderContext, targets: &[RenderTarget]) -> Texture {
        // Use the first target for dimensions, assuming all targets have the same size
        // You might want to add validation that all targets have the same dimensions
        if targets.is_empty() {
            panic!("Cannot create depth texture: no render targets specified");
        }

        let (width, height) = targets[0].dimensions(render_ctx);
        render_ctx.create_texture(TextureCreateInfo {
            label: Some("depth texture"),
            width,
            height,
            format: crate::core::texture::TextureFormat::Depth32,
            usage: TextureUsage::RENDER_ATTACHMENT,
        })
    }

    /// recreates the depth texture if its auto
    fn recreate_depth_texture(&mut self, render_ctx: &RenderContext) {
        if let DepthMode::Auto(depth_options) = &mut self.depth {
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

pub enum DepthTarget {
    /// no depth buffer
    None,
    /// depth buffer matches render target
    Auto {
        compare_function: DepthCompare,
        depth_bias: Option<(f32, f32)>,
    },
    /// specify a texture to render depth too
    Texture {
        depth_texture: Texture,
        compare_function: DepthCompare,
        depth_bias: Option<(f32, f32)>,
    },
}

pub enum DepthMode {
    None,
    Auto(DepthStencilOptions),
    Manual(DepthStencilOptions),
}

impl DepthMode {
    pub fn map_to_option(&self) -> Option<&DepthStencilOptions> {
        match self {
            DepthMode::None => None,
            DepthMode::Manual(options) => Some(options),
            DepthMode::Auto(options) => Some(options),
        }
    }
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
    pub target: Vec<RenderTarget>,
    pub depth: DepthTarget,
    pub cull_mode: CullMode,
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
        color_format: Option<TextureFormat>,
        info: RenderNodeDescriptor,
    ) -> Self {
        let depth = match info.depth {
            DepthTarget::None => DepthMode::None,
            DepthTarget::Auto {
                compare_function,
                depth_bias,
            } => {
                let depth_texture =
                    RenderNodeContext::create_depth_texture(render_ctx, &info.target);

                DepthMode::Auto(DepthStencilOptions {
                    texture: depth_texture,
                    compare: compare_function,
                    write_enabled: true,
                    depth_bias,
                })
            }
            DepthTarget::Texture {
                depth_texture,
                compare_function,
                depth_bias,
            } => DepthMode::Manual(DepthStencilOptions {
                texture: depth_texture,
                compare: compare_function,
                write_enabled: true,
                depth_bias,
            }),
        };

        // Only create a pipeline if the node has at least one render target (color or depth)
        // Resource-only nodes don't need pipelines
        let pipeline = if color_format.is_some() || !matches!(depth, DepthMode::None) {
            let pipeline_layout = render_ctx.create_pipeline_layout(&info.descriptor_set_layouts);

            Some(render_ctx.create_pipeline(PipelineCreateInfo {
                label: pipeline_label,
                shader: info.shader.clone(),
                layout: pipeline_layout,
                color_format,
                depth: &depth,
                cull_mode: info.cull_mode,
            }))
        } else {
            None
        };

        let ctx = RenderNodeContext {
            shader: info.shader,
            pipeline,
            target: info.target,
            depth,
        };

        RenderNodeWrapper { context: ctx, pass }
    }

    pub fn resize(&mut self, render_ctx: &RenderContext, dimensions: [u32; 2]) {
        if self
            .context
            .target()
            .iter()
            .any(|t| matches!(t, RenderTarget::Surface))
            && let DepthMode::Auto(_) = &self.context.depth
        {
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
            target: vec![],
            depth: DepthTarget::None,
            cull_mode: CullMode::Back,
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
