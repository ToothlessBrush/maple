use std::{error::Error, marker::PhantomData, rc::Rc, sync::Arc};

use anyhow::Result;
use maple_engine::{components::Event, context::GameContext, scene::SceneBuilder};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Fullscreen, Window, WindowId},
};

use maple_renderer::{core::renderer::Renderer, types::render_config::RenderConfig};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::{
    app_error::AppError,
    config::{Config, WindowMode},
    plugin::Plugin,
};

// ============================================================================
// App States
// ============================================================================

/// Init app state where you can load plugins/scenes but can't reference the renderer etc
pub struct Init;

/// Running app state where the app is in the event loop. The renderer is initialized in this state
pub struct Running;

// ============================================================================
// Runtime State
// ============================================================================

/// Contains the runtime state of the application (window, renderer, etc.)
pub struct AppState {
    window: Arc<Window>,
    renderer: Renderer,
}

impl AppState {
    pub fn new(window: Arc<Window>, renderer: Renderer) -> Self {
        Self { window, renderer }
    }

    pub fn window(&self) -> &Arc<Window> {
        &self.window
    }

    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    pub fn renderer_mut(&mut self) -> &mut Renderer {
        &mut self.renderer
    }

    pub fn draw(&mut self) -> Result<()> {
        self.renderer.begin_draw().map_err(|e| {
            eprintln!("Failed to render: {e}");
            e
        })
    }

    pub fn resize(&mut self, new_size: [u32; 2]) {
        self.renderer.resize(new_size);
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }
}

/// Main app for the engine
///
/// This handles the window and event loop
pub struct App<S = Init> {
    state: Option<AppState>,
    context: GameContext,
    config: Config,
    plugins: Vec<Rc<dyn Plugin>>,
    _app_state: PhantomData<S>,
}

impl Default for App<Init> {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl App<Init> {
    /// Creates a new app with the given configuration
    pub fn new(config: Config) -> Self {
        Self {
            state: None,
            plugins: Vec::new(),
            context: GameContext::default(),
            config,
            _app_state: PhantomData,
        }
    }

    /// Loads a scene into the app
    pub fn load_scene<T: SceneBuilder>(mut self, scene: T) -> Self {
        self.context.scene.load(scene);
        self
    }

    /// Adds a plugin to the app
    pub fn add_plugin<T: Plugin + 'static>(mut self, plugin: T) -> Self {
        self.plugins.push(Rc::new(plugin));
        self
    }

    /// Runs the application
    ///
    /// This will block as long as the window is open, so call this last
    pub fn run(self) -> Result<(), AppError> {
        env_logger::init();

        let mut initialized_app = self.transition_to_running();
        let event_loop = EventLoop::new()?;

        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(&mut initialized_app)?;

        Ok(())
    }

    /// Transitions the app from Init to Running state
    fn transition_to_running(self) -> App<Running> {
        App::<Running> {
            state: None, // State is initialized inside of resume
            plugins: self.plugins,
            context: self.context,
            config: self.config,
            _app_state: PhantomData,
        }
    }
}

impl App<Running> {
    /// Gets a reference to the renderer
    ///
    /// # Panics
    /// Panics if the app state hasn't been initialized yet
    pub fn renderer(&self) -> &Renderer {
        self.state().renderer()
    }

    /// Gets a mutable reference to the renderer
    ///
    /// # Panics
    /// Panics if the app state hasn't been initialized yet
    pub fn renderer_mut(&mut self) -> &mut Renderer {
        self.state_mut().renderer_mut()
    }

    /// Gets a reference to the window
    ///
    /// # Panics
    /// Panics if the app state hasn't been initialized yet
    pub fn window(&self) -> &Arc<Window> {
        self.state().window()
    }

    /// Gets the app config
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Gets the game context
    pub fn context(&self) -> &GameContext {
        &self.context
    }

    /// Gets a mutable reference to the game context
    pub fn context_mut(&mut self) -> &mut GameContext {
        &mut self.context
    }

    fn state(&self) -> &AppState {
        self.state.as_ref().expect("App state not initialized")
    }

    fn state_mut(&mut self) -> &mut AppState {
        self.state.as_mut().expect("App state not initialized")
    }

    fn create_window(&self, event_loop: &ActiveEventLoop) -> Result<Arc<Window>, AppError> {
        let window_attributes = self.build_window_attributes();
        let window = event_loop.create_window(window_attributes)?;
        Ok(Arc::new(window))
    }

    fn build_window_attributes(&self) -> winit::window::WindowAttributes {
        let mut attributes = Window::default_attributes()
            .with_title(self.config.window_title)
            .with_resizable(self.config.resizeable)
            .with_decorations(self.config.decorated)
            .with_fullscreen(self.get_fullscreen_mode());

        if let Some(resolution) = &self.config.resolution {
            attributes = attributes.with_inner_size(resolution.physical_size());
        }

        attributes
    }

    fn get_fullscreen_mode(&self) -> Option<Fullscreen> {
        match self.config.window_mode {
            WindowMode::Windowed => None,
            WindowMode::Borderless => Some(Fullscreen::Borderless(None)),
            WindowMode::FullScreen => {
                // TODO: Implement exclusive video mode selection
                Some(Fullscreen::Borderless(None))
            }
        }
    }

    fn create_renderer(&self, window: Arc<Window>) -> Renderer {
        let renderer_config = RenderConfig {
            vsync: self.config.vsync,
            dimensions: window.inner_size().into(),
        };

        match Renderer::init(window, renderer_config) {
            Ok(renderer) => renderer,
            Err(e) => {
                eprintln!("Failed to initialize renderer, running in headless mode: {e}");
                Renderer::headless()
            }
        }
    }

    fn initialize_plugins(&mut self) {
        let plugins = std::mem::take(&mut self.plugins);

        for plugin in &plugins {
            plugin.init(self);
        }

        self.plugins = plugins;
    }

    fn update_plugins(&mut self) {
        let plugins = std::mem::take(&mut self.plugins);

        for plugin in &plugins {
            plugin.update(self);
        }

        self.plugins = plugins;
    }

    fn initialize_app_state(&mut self, event_loop: &ActiveEventLoop) -> Result<(), AppError> {
        let window = self.create_window(event_loop)?;
        let renderer = self.create_renderer(window.clone());
        self.state = Some(AppState::new(window, renderer));
        Ok(())
    }

    fn handle_frame(&mut self) {
        self.context.begin_frame();

        self.context.emit(Event::Update);
        self.update_plugins();

        self.context.end_frame();
    }
}

impl ApplicationHandler for App<Running> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return; // Already initialized
        }

        match self.initialize_app_state(event_loop) {
            Ok(()) => {
                self.initialize_plugins();
                self.context.emit(Event::Ready);
            }
            Err(e) => {
                eprintln!("Failed to initialize app: {e}");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // Forward event to context
        self.context.window_event(&event);

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.state_mut().resize(size.into());
            }
            WindowEvent::RedrawRequested => {
                self.handle_frame();
                self.state().request_redraw();
            }
            _ => {}
        }
    }
}
