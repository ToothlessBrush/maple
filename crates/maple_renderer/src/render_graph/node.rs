//! a single pass which exists within the graph

use maple_engine::GameContext;

use crate::{
    core::{Frame, RenderContext},
    platform::SendSync,
    render_graph::graph::{RenderGraphContext, Stage},
    types::Dimensions,
};

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
