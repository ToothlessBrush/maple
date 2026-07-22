use maple_engine::GameContext;

use crate::{
    core::{
        DepthCompare, DepthStencilOptions, Frame, RenderContext,
        texture::{Texture, TextureView},
    },
    platform::SendSync,
    render_graph::graph::{RenderGraphContext, Stage},
    types::Dimensions,
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

#[derive(PartialEq, Debug, Clone)]
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

pub trait RenderNode: SendSync {
    fn stage(&self) -> Stage;

    fn label() -> &'static str
    where
        Self: Sized,
    {
        ""
    }

    fn setup(rcx: &RenderContext, graph_ctx: &mut RenderGraphContext) -> Self
    where
        Self: Sized;

    /// called every frame here is where you put logic to draw stuff
    fn draw(
        &mut self,
        renderer_ctx: &RenderContext,
        frame: &mut Frame,
        graph_ctx: &mut RenderGraphContext,
        game_ctx: &GameContext,
    );

    /// called when the window resizes if that is relavent to the pass
    #[allow(unused)]
    fn resize(&mut self, render_ctx: &RenderContext, dimensions: Dimensions) {}
}
