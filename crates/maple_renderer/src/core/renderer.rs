use anyhow::Result;
use maple_engine::Scene;

use std::{error::Error, sync::Arc};

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::{
    core::RenderContext,
    render_graph::{
        graph::{GraphBuilder, RenderGraph},
        node::{RenderNode, RenderNodeWrapper},
    },
    types::render_config::RenderConfig,
};

// TODO create a render context to avoid passing itself to the graph

/// The Renderer handles all rendering tasks for the engine as well as provides tools to help in
/// pass creation
pub struct Renderer {
    pub context: RenderContext,
    pub render_graph: RenderGraph,
}

impl Renderer {
    /// creates and initializes the renderer
    pub fn init<T>(window: Arc<T>, config: RenderConfig) -> Result<Self>
    where
        T: HasWindowHandle + HasDisplayHandle + Send + Sync + 'static,
    {
        let context = pollster::block_on(RenderContext::init(window, config))?;

        Ok(Renderer {
            context,
            render_graph: RenderGraph::default(),
        })
    }

    /// resize the surface as well as render_passes that might need that
    pub fn resize(&mut self, dimensions: [u32; 2]) {
        self.context.resize(dimensions);
        self.render_graph.resize(&self.context, dimensions);
    }

    pub fn graph(&mut self) -> GraphBuilder<'_> {
        GraphBuilder::create(self)
    }

    /// begins the render passes within the render graph patent pending
    pub fn begin_draw(&mut self, scene: &Scene) -> Result<(), Box<dyn Error>> {
        self.context.acquire_surface_texture()?;

        self.render_graph.render(&self.context, scene)?;

        self.context.present_surface()?;

        Ok(())
    }

    /// add a node to the render graph
    pub(crate) fn setup_render_node<T>(
        &mut self,
        mut node: T,
    ) -> RenderNodeWrapper
    where
        T: RenderNode + 'static,
    {
        node.setup(&self.context, &mut self.render_graph.context);
        RenderNodeWrapper::create(Box::new(node))
    }
}
