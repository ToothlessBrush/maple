use maple_engine::Scene;

use crate::{
    core::{
        DepthCompare, DepthStencilOptions, RenderContext,
        texture::{Texture, TextureView},
    },
    render_graph::graph::RenderGraphContext,
};

#[derive()]
pub enum RenderTarget {
    Surface,
    Texture(TextureView),
    MultiSampled {
        texture: TextureView,
        resolve: TextureView,
    },
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
    Texture(DepthStencilOptions),
}

impl DepthMode {
    pub fn map_to_option(&self) -> Option<&DepthStencilOptions> {
        match self {
            DepthMode::None => None,
            DepthMode::Texture(options) => Some(options),
        }
    }
}

impl RenderTarget {
    // /// gets the dimensions of the target  (width, height)
    // pub fn dimensions(&self, render_ctx: &RenderContext) -> (u32, u32) {
    //     match self {
    //         RenderTarget::Surface => render_ctx.surface_size(),
    //         RenderTarget::Texture(tex) => (tex.width(), tex.height()),
    //         RenderTarget::MultiSampled {
    //             texture,
    //             resolve: _,
    //         } => (texture.width(), texture.height()),
    //     }
    // }
}

pub trait RenderNode {
    /// sets up the renderpass here is where you compile shaders, set up descritors, etc...
    fn setup(&mut self, render_ctx: &RenderContext, graph_ctx: &mut RenderGraphContext);

    /// called every frame here is where you put logic to draw stuff
    fn draw(
        &mut self,
        renderer_ctx: &RenderContext,
        graph_ctx: &mut RenderGraphContext,
        scene: &Scene,
    );

    /// called when the window resizes if that is relavent to the pass
    #[allow(unused)]
    fn resize(&mut self, render_ctx: &RenderContext, dimensions: [u32; 2]) {}
}

pub(crate) struct RenderNodeWrapper {
    pub pass: Box<dyn RenderNode>,
}

impl RenderNodeWrapper {
    pub fn create(pass: Box<dyn RenderNode>) -> Self {
        RenderNodeWrapper { pass }
    }

    pub fn resize(&mut self, render_ctx: &RenderContext, dimensions: [u32; 2]) {
        self.pass.resize(render_ctx, dimensions);
    }
}

pub struct Marker;

impl RenderNode for Marker {
    fn setup(&mut self, _render_ctx: &RenderContext, _graph_ctx: &mut RenderGraphContext) {}

    fn draw(
        &mut self,
        _renderer_ctx: &RenderContext,
        _graph_ctx: &mut RenderGraphContext,
        _scene: &Scene,
    ) {
    }
}
