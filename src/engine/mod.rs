use egui_backend::glfw;
use egui_gl_glfw as egui_backend;
use egui_gl_glfw::glfw::Context;

use renderer::Renderer;

pub mod game_context;
pub mod renderer;
pub mod utils;

use game_context::GameContext;

pub struct Engine {
    pub window: glfw::PWindow,

    pub context: GameContext,
}

const SAMPLES: u32 = 8;

impl Engine {
    pub fn init(window_title: &str, window_width: u32, window_height: u32) -> Engine {
        use glfw::fail_on_errors;
        let mut glfw = glfw::init(fail_on_errors!()).unwrap();
        glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));
        glfw.window_hint(glfw::WindowHint::DoubleBuffer(true));
        glfw.window_hint(glfw::WindowHint::Resizable(false));
        glfw.window_hint(glfw::WindowHint::Samples(Some(SAMPLES)));

        let (mut window, events) = glfw
            .create_window(
                window_width,
                window_height,
                window_title,
                glfw::WindowMode::Windowed,
            )
            .expect("Failed to create GLFW window.");

        //set up input polling
        window.set_key_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_mouse_button_polling(true);
        window.set_scroll_polling(true);
        window.set_framebuffer_size_polling(true);
        window.make_current();

        //load grahpics api
        Renderer::context(&mut window);

        Renderer::init();

        Engine {
            window,
            context: GameContext::new(events, glfw),
        }
    }

    pub fn set_clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        Renderer::set_clear_color([r, g, b, a]);
    }

    pub fn begin(&mut self) {
        for model in self.context.nodes.borrow_mut().models.values_mut() {
            model.ready();
        }

        for camera in self.context.nodes.borrow_mut().cameras.values_mut() {
            camera.ready();
        }

        if self.context.nodes.borrow().active_camera.is_empty() {
            eprintln!("Warning: No camera found in the scene");
        }

        if self.context.nodes.borrow().active_shader.is_empty() {
            eprintln!("Warning: No shader found in the scene");
        }

        //render loop
        self.render_loop();
    }

    fn render_loop(&mut self) {
        while !self.window.should_close() {
            Renderer::clear();

            self.context.frame.borrow_mut().update(|fps| {
                self.window.set_title(&format!("FPS: {}", fps));
            });
            self.context.input.borrow_mut().update();

            for (_, ui) in self.context.nodes.borrow_mut().uis.iter_mut() {
                ui.update(&self.context);
            }

            for (_, model) in self.context.nodes.borrow_mut().models.iter_mut() {
                model.behavior(&self.context);
            }

            for (_, camera) in self.context.nodes.borrow_mut().cameras.iter_mut() {
                camera.behavior(&self.context);
            }

            for (_, model) in self.context.nodes.borrow_mut().models.iter_mut() {
                if let Some(shader) = self
                    .context
                    .nodes
                    .borrow_mut()
                    .shaders
                    .get_mut(&self.context.nodes.borrow_mut().active_shader)
                {
                    model.draw(
                        shader,
                        &self.context.nodes.borrow_mut().cameras
                            [&self.context.nodes.borrow_mut().active_camera],
                    );
                }
            }

            for (_, ui) in self.context.nodes.borrow_mut().uis.iter_mut() {
                ui.render(&self.context);
            }

            self.window.swap_buffers();
        }
    }
}
