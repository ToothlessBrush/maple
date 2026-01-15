use anyhow::Result;
use maple_engine::Scene;

use std::{error::Error, sync::Arc};

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::{
    core::RenderContext,
    render_graph::graph::{GraphBuilder, RenderGraph},
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
    /// creates and initializes the renderer (blocking, for native platforms)
    #[cfg(not(target_arch = "wasm32"))]
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

    /// creates and initializes the renderer (async, for wasm platforms)
    #[cfg(target_arch = "wasm32")]
    pub async fn init_async<T>(window: Arc<T>, config: RenderConfig) -> Result<Self>
    where
        T: HasWindowHandle + HasDisplayHandle + 'static,
    {
        let context = RenderContext::init(window, config).await?;
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
}
