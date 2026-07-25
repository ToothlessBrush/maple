//! a single pass which exists within the graph

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

/// target of the render pass where the image will be drawn
#[derive()]
pub enum RenderTarget {
    /// surface texture used by the window
    Surface,
    /// a 2d texture
    Texture(TextureView),
    /// MSAA sampled texture
    MultiSampled {
        /// MSAA texture
        texture: TextureView,
        /// MSAA resolve texture
        resolve: TextureView,
    },
}

/// Target the render pass uses to draw depth
pub enum DepthTarget {
    /// no depth buffer
    None,
    /// depth buffer matches render target
    Auto {
        /// compare function used
        compare_function: DepthCompare,
        /// bias values used
        depth_bias: Option<(f32, f32)>,
    },
    /// specify a texture to render depth too
    Texture {
        /// depth texture
        depth_texture: Texture,
        /// compare function used
        compare_function: DepthCompare,
        /// bias values used
        depth_bias: Option<(f32, f32)>,
    },
}

/// mode for depth writing
#[derive(PartialEq, Debug, Clone)]
pub enum DepthMode {
    /// no depth mode
    None,
    /// texture mode of depth
    Texture(DepthStencilOptions),
}

impl DepthMode {
    /// maps the depth mod to a [`Option`]
    pub fn map_to_option(&self) -> Option<&DepthStencilOptions> {
        match self {
            DepthMode::None => None,
            DepthMode::Texture(options) => Some(options),
        }
    }
}

/// exists within the render graph and provides methods for setting up and drawing
pub trait RenderNode: SendSync {
    /// when in the render pass this is ran
    fn stage(&self) -> Stage;

    /// optional label for the pass
    fn label() -> &'static str
    where
        Self: Sized,
    {
        ""
    }

    /// ran when added setup things like Shader or Pipelines
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
