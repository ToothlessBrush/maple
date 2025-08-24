use std::{
    marker::PhantomData,
    rc::Rc,
    sync::{Arc, Mutex, MutexGuard},
};

use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalSize, Size},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use maple_renderer::core::{
    render_pass::RenderPass,
    renderer::{self, Renderer},
};

use crate::plugin::Plugin;

// app states
/// uninitialized app state where you can load plugins/scenes but cant refrence the renderer etc
pub struct Uninitialized;
/// Running app state where the app is in the event loop. the renderer is initialized in this state
pub struct Running;

/// main app for the engine
///
/// this handles the window and event loop
#[derive(Default)]
pub struct App<State = Uninitialized> {
    window: Option<Arc<Window>>,
    renderer: Renderer,
    plugins: Vec<Rc<dyn Plugin>>,
    state: PhantomData<State>,
}

impl ApplicationHandler for App<Running> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Hello, Window!")
                        .with_inner_size(PhysicalSize {
                            width: 1920,
                            height: 1080,
                        }),
                )
                .unwrap(),
        );

        self.renderer = match Renderer::init(window.clone(), window.inner_size().into()) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("failed to init renderer running in headless mode: {e}");
                Renderer::default()
            }
        };

        self.window = Some(window);

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
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => self.renderer.resize(size.into()),
            WindowEvent::RedrawRequested => {
                // call the draw function
                self.draw();

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

impl App {
    /// creates an app
    pub fn new() -> App<Uninitialized> {
        Self {
            window: None,
            renderer: Renderer::default(),
            plugins: Vec::new(),
            state: std::marker::PhantomData,
        }
    }
}

impl App<Uninitialized> {
    /// runs the application
    ///
    /// this will block as long as the window is open so call this last
    pub fn run(&mut self) {
        // switch app to running state
        let mut initialized_app = App::<Running> {
            renderer: Renderer::default(),
            window: None,
            plugins: std::mem::take(&mut self.plugins),
            state: PhantomData::<Running>,
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
        self.renderer.add_pass(pass);
        self
    }

    /// called everytime a frame draw is requested
    fn draw(&mut self) {
        self.renderer.draw();
    }
}
