//! base struct of the renderer handles initialization and render graph dispatching

use maple_engine::{GameContext, platform::SendSync};
use wgpu::CreateSurfaceError;

use std::sync::Arc;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::core::context::InitError;
use crate::{
    core::{RenderContext, context::SurfaceError},
    render_graph::graph::{GraphBuilder, RenderGraph},
    types::{Dimensions, render_config::RenderConfig},
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
    pub fn init<T>(window: Arc<T>, config: RenderConfig) -> Result<Self, InitError>
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
    pub async fn init<T>(window: Arc<T>, config: RenderConfig) -> Result<Self, InitError>
    where
        T: HasWindowHandle + HasDisplayHandle + 'static,
    {
        let context = RenderContext::init(window, config).await?;
        Ok(Renderer {
            context,
            render_graph: RenderGraph::default(),
        })
    }

    /// creates and initializes the renderer (blocking, for native platforms)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn init_headless(config: RenderConfig) -> Result<Self, InitError> {
        let context = pollster::block_on(RenderContext::init_headless(config))?;
        Ok(Renderer {
            context,
            render_graph: RenderGraph::default(),
        })
    }

    /// creates and initializes the renderer (async, for wasm platforms)
    #[cfg(target_arch = "wasm32")]
    pub async fn init_headless(config: RenderConfig) -> Result<Self, InitError> {
        let context = RenderContext::init_headless(config).await?;
        Ok(Renderer {
            context,
            render_graph: RenderGraph::default(),
        })
    }

    pub fn attach_surface<T>(
        &mut self,
        window: Arc<T>,
        dimensions: Dimensions,
    ) -> Result<(), CreateSurfaceError>
    where
        T: HasDisplayHandle + HasWindowHandle + SendSync + 'static,
    {
        self.context.attach_surface(window, dimensions)
    }

    /// resize the surface as well as render_passes that might need that
    pub fn resize(&mut self, dimensions: Dimensions) {
        self.context.resize(dimensions);
        self.render_graph.resize(&self.context, dimensions);
    }

    pub fn graph(&mut self) -> GraphBuilder<'_> {
        GraphBuilder::create(self)
    }

    /// begins the render passes within the render graph patent pending
    pub fn draw(&mut self, ctx: &GameContext) {
        let texture = match self.context.acquire_surface_texture() {
            Ok(surface) => surface,
            Err(err) => match err {
                SurfaceError::Occluded
                | SurfaceError::Timeout
                | SurfaceError::Outdated
                | SurfaceError::Validation => {
                    log::warn!("frame skipped due to: {err}");
                    return;
                }
                SurfaceError::SurfaceMissing => {
                    log::error!("tried to draw without an attached surface: {err}");
                    return;
                }
                SurfaceError::ContextLost => {
                    panic!("context lost: {err}");
                }
            },
        };

        let mut frame = self.context.create_frame(texture);

        self.render_graph.render(&self.context, ctx, &mut frame);

        let surface_texture = self.context.submit_frame(frame);

        self.context.present_surface(surface_texture);
    }
}
