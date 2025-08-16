use std::{
    rc::Rc,
    sync::{Arc, Mutex, MutexGuard},
};

use winit::{
    application::ApplicationHandler,
    dpi::{PhysicalSize, Size},
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use maple_renderer::core::{render_pass::RenderPass, renderer::Renderer};

use crate::plugin::Plugin;

pub struct Uninitialized;
pub struct Initialized;

/// main app for the engine
///
/// this handles the window and event loop
#[derive(Default)]
pub struct App<State = Uninitialized> {
    window: Option<Arc<Window>>,
    renderer: Renderer,
    plugins: Vec<Rc<dyn Plugin>>,
    state: std::marker::PhantomData<State>,
}

impl ApplicationHandler for App<Initialized> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Hello, Window!")
                        .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
                        .with_inner_size(PhysicalSize {
                            width: 2560,
                            height: 1440,
                        })
                        .with_resizable(false),
                )
                .unwrap(),
        );

        self.renderer = Renderer::init(window.clone(), window.inner_size().into());

        self.window = Some(window);

        let plugins = std::mem::take(&mut self.plugins);

        for plugin in &plugins {
            plugin.init(self)
        }

        self.plugins = plugins;
    }
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.renderer
                    .resize(size.into())
                    .expect("failed to resize buffer");
            }
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
        let mut initialized_app = App::<Initialized> {
            renderer: Renderer::default(),
            window: None,
            plugins: std::mem::take(&mut self.plugins),
            state: std::marker::PhantomData::<Initialized>,
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

impl App<Initialized> {
    pub fn add_renderpass<T: RenderPass + 'static>(&mut self, pass: T) -> &mut Self {
        if let Err(e) = self.renderer.add_pass(pass) {
            eprintln!("failed to add render pass: {e}");
        }

        self
    }

    /// called everytime a frame draw is requested
    fn draw(&mut self) {
        self.renderer.draw();
    }
}
