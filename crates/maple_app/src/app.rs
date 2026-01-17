use anyhow::Result;
use log::error;
use maple_engine::{Scene, context::GameContext, prelude::Frame};
use std::{marker::PhantomData, process, rc::Rc, sync::Arc};
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
    default_plugin::DefaultPlugin,
    plugin::Plugin,
};

/// Init app state where you can load plugins/scenes but can't reference the renderer etc
pub struct Init;

/// Running app state where the app is in the event loop. The renderer is initialized in this state
pub struct Running;

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

    pub fn resize(&mut self, new_size: [u32; 2]) {
        self.renderer.resize(new_size);
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub(crate) fn draw(&mut self, scene: &Scene) {
        // TODO: Create Complete Render Error for runtime Render Errors
        self.renderer
            .begin_draw(scene)
            .expect("Failed to draw scene");
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
    #[cfg(target_arch = "wasm32")]
    pending_renderer: Option<(
        Arc<Window>,
        std::rc::Rc<std::cell::RefCell<Option<Renderer>>>,
    )>,
    _marker: PhantomData<S>,
}

impl Default for App<Init> {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl App<Init> {
    /// Creates a new app with the given configuration
    pub fn new(config: Config) -> Self {
        // add core resources
        let ctx = GameContext::default();

        Self {
            state: None,
            plugins: Vec::new(),
            context: ctx,
            config,
            #[cfg(target_arch = "wasm32")]
            pending_renderer: None,
            _marker: PhantomData,
        }
        .add_plugin(DefaultPlugin)
    }

    /// Loads a scene into the app
    pub fn load_scene<T>(self, scene: T) -> Self
    where
        T: Into<Scene>,
    {
        self.context.scene.merge(scene.into());
        self
    }

    /// Get access to the context during initialization
    pub fn context(&self) -> &GameContext {
        &self.context
    }

    /// Get mutable access to the context during initialization
    pub fn context_mut(&mut self) -> &mut GameContext {
        &mut self.context
    }

    /// Adds a plugin to the app
    pub fn add_plugin<T: Plugin + 'static>(mut self, plugin: T) -> Self {
        let plugin_rc = Rc::new(plugin);

        // Call setup immediately during Init phase
        plugin_rc.setup(&mut self);

        self.plugins.push(plugin_rc);
        self
    }

    /// Runs the application
    ///
    /// This will block as long as the window is open, so call this last
    pub fn run(self) {
        let mut initialized_app = self.transition_to_running();

        let event_loop = match EventLoop::new() {
            Ok(event_loop) => event_loop,
            Err(e) => {
                error!("Fatal Error: Event loop failed to initialize: {e}");
                error!("Is windowing available?");
                process::exit(1);
            }
        };

        event_loop.set_control_flow(ControlFlow::Poll);

        if let Err(e) = event_loop.run_app(&mut initialized_app) {
            error!("Fatal Error: Event loop execution failed: {e}");
            process::exit(1);
        };
    }

    /// Transitions the app from Init to Running state
    fn transition_to_running(self) -> App<Running> {
        App::<Running> {
            state: None, // State is initialized inside of resume
            plugins: self.plugins,
            context: self.context,
            config: self.config,
            #[cfg(target_arch = "wasm32")]
            pending_renderer: None,
            _marker: PhantomData,
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

    #[cfg(not(target_arch = "wasm32"))]
    fn create_renderer(&self, window: Arc<Window>) -> Renderer {
        let renderer_config = RenderConfig {
            vsync: self.config.vsync,
            dimensions: window.inner_size().into(),
        };

        Renderer::init(window, renderer_config)
            .expect("Failed to initialize renderer. Cannot continue without a renderer.")
    }

    fn initialize_plugins(&mut self) {
        let plugins = std::mem::take(&mut self.plugins);

        for plugin in &plugins {
            plugin.ready(self);
        }

        self.plugins = plugins;

        self.context().scene.sync_world_transform();
    }

    fn update_plugins(&mut self) {
        let plugins = std::mem::take(&mut self.plugins);

        for plugin in &plugins {
            plugin.update(self);
        }

        self.plugins = plugins;

        // sync worlds after plugins may have changed transforms
        self.context().scene.sync_world_transform();
    }

    fn fixed_update_plugins(&mut self) {
        let plugins = std::mem::take(&mut self.plugins);

        for plugin in &plugins {
            plugin.fixed_update(self);
        }

        self.plugins = plugins;

        self.context().scene.sync_world_transform();
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn initialize_app_state(&mut self, event_loop: &ActiveEventLoop) -> Result<(), AppError> {
        let window = self.create_window(event_loop)?;
        let renderer = self.create_renderer(window.clone());
        self.state = Some(AppState::new(window, renderer));
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    fn initialize_app_state(&mut self, event_loop: &ActiveEventLoop) -> Result<(), AppError> {
        use winit::platform::web::WindowExtWebSys;

        let window = self.create_window(event_loop)?;
        let canvas = window.canvas().expect("Failed to get canvas");

        let size = window.inner_size();
        canvas.set_width(size.width);
        canvas.set_height(size.height);

        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| body.append_child(&canvas).ok())
            .expect("Failed to append canvas to body");

        let window_clone = window.clone();
        let vsync = self.config.vsync;
        let dimensions = window.inner_size().into();

        // Create a shared cell for the renderer
        let renderer_cell = std::rc::Rc::new(std::cell::RefCell::new(None));
        let renderer_cell_clone = renderer_cell.clone();

        // Spawn async renderer initialization
        wasm_bindgen_futures::spawn_local(async move {
            let renderer_config = RenderConfig { vsync, dimensions };

            match Renderer::init_async(window_clone.clone(), renderer_config).await {
                Ok(renderer) => {
                    log::info!("Renderer initialized successfully");
                    *renderer_cell_clone.borrow_mut() = Some(renderer);
                    window_clone.request_redraw();
                }
                Err(e) => {
                    log::error!("Failed to initialize renderer: {}", e);
                }
            }
        });

        // Store window and renderer cell so we can check it later
        self.pending_renderer = Some((window, renderer_cell));
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    fn check_pending_renderer(&mut self) {
        if let Some((window, renderer_cell)) = self.pending_renderer.take() {
            if let Some(renderer) = renderer_cell.borrow_mut().take() {
                // Renderer is ready, initialize state
                self.state = Some(AppState::new(window, renderer));
                self.initialize_plugins();
            } else {
                // Not ready yet, put it back
                self.pending_renderer = Some((window, renderer_cell));
            }
        }
    }

    fn draw(&mut self) {
        if let Some(state) = self.state.as_mut() {
            state.draw(&self.context.scene);
        }
    }

    /// This is called everyframe
    ///
    /// called from the winit requested redraw event
    fn handle_frame(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            // Check if renderer is ready
            self.check_pending_renderer();

            // If state is still not ready, return early
            if self.state.is_none() {
                return;
            }
        }

        self.context.begin_frame();

        // Run fixed update as many times as needed based on accumulated time
        while self
            .context
            .get_resource_mut::<Frame>()
            .should_fixed_update()
        {
            self.fixed_update_plugins();
        }

        self.update_plugins();

        self.draw();

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
                // For native, initialize plugins immediately
                // For WASM, plugins are initialized when renderer is ready
                #[cfg(not(target_arch = "wasm32"))]
                self.initialize_plugins();
            }
            Err(e) => {
                log::error!("Failed to initialize app: {e}");
                event_loop.exit();
            }
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        self.context.device_event(&event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        #[cfg(target_arch = "wasm32")]
        {
            // On WASM, check if renderer is ready before processing any events
            self.check_pending_renderer();

            // Skip all events until state is initialized
            if self.state.is_none() {
                return;
            }
        }
        self.context.window_event(&event);

        match event {
            WindowEvent::CloseRequested => {
                log::info!("Bye o/");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.state_mut().resize(size.into());
            }
            WindowEvent::RedrawRequested => {
                self.handle_frame();
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(state) = &self.state {
            state.request_redraw();
        }
    }
}
