use std::{marker::PhantomData, rc::Rc, sync::Arc};

use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use maple_renderer::core::{render_pass::RenderPass, renderer::Renderer};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::plugin::Plugin;

// app states
/// Init app state where you can load plugins/scenes but cant refrence the renderer etc
pub struct Init;
/// Running app state where the app is in the event loop. the renderer is initialized in this state
pub struct Running;

pub struct State {
    window: Arc<Window>,
    renderer: Renderer,
}

impl State {
    pub fn draw(&mut self) {
        self.renderer.draw();
    }
}

/// main app for the engine
///
/// this handles the window and event loop
#[derive(Default)]
pub struct App<S = Init> {
    state: Option<State>,
    plugins: Vec<Rc<dyn Plugin>>,
    _app_state: PhantomData<S>,
}

impl ApplicationHandler for App<Running> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let window_attributes = Window::default_attributes();

        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .expect("failed to create window"),
        );

        let renderer = match Renderer::init(window.clone(), window.inner_size().into()) {
            Ok(r) => r,
            Err(e) => {
                eprintln!(
                    "failed to init renderer, running in headless mode. brace for impact: {e}"
                );
                Renderer::headless()
            }
        };

        let state = State { window, renderer };

        self.state = Some(state);

        let plugins = std::mem::take(&mut self.plugins);

        for plugin in &plugins {
            plugin.init(self)
        }

        self.plugins = plugins;
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(state) = &mut self.state else {
            eprintln!("engine state has not been created (something went really wrong)");
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => state.renderer.resize(size.into()),
            WindowEvent::RedrawRequested => {
                // call the draw function
                state.draw();

                state.window.request_redraw();
            }
            _ => {}
        }
    }
}

impl App {
    /// creates an app
    pub fn new() -> App<Init> {
        Self {
            state: None,
            plugins: Vec::new(),
            _app_state: std::marker::PhantomData,
        }
    }
}

impl App<Init> {
    /// runs the application
    ///
    /// this will block as long as the window is open so call this last
    pub fn run(&mut self) {
        env_logger::init();

        // switch app to running state
        let mut initialized_app = App::<Running> {
            state: None, // state in initialized inside of resume
            plugins: std::mem::take(&mut self.plugins),
            _app_state: PhantomData::<Running>,
        };

        let event_loop = EventLoop::new().unwrap();

        event_loop.set_control_flow(ControlFlow::Poll);

        match event_loop.run_app(&mut initialized_app) {
            Ok(_) => {}
            Err(e) => {
                eprint!("app failed while running: {e}")
            }
        }
    }

    pub fn add_plugin<T: Plugin + 'static>(&mut self, plugin: T) -> &mut Self {
        self.plugins.push(Rc::new(plugin));

        self
    }
}

impl App<Running> {
    pub fn add_renderpass<T: RenderPass + 'static>(&mut self, pass: T) -> &mut Self {
        let Some(state) = &mut self.state else {
            eprintln!("state has not been initialized (something went really wrong)");
            return self;
        };

        state.renderer.add_render_node(pass);

        self
    }
}
