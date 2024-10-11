use egui_backend::egui;
use egui_backend::glfw;
use egui_gl_glfw as egui_backend;
use egui_gl_glfw::glfw::Context;
use gltf::camera;

use std::collections::HashMap;

use renderer::Renderer;

pub mod nodes;
pub mod renderer;
pub mod utils;

use nodes::{camera::Camera3D, model::Model, ui::UI};
use renderer::shader::Shader;

use utils::fps_manager::FPSManager;
use utils::input_manager::InputManager;

pub struct Engine {
    window: glfw::PWindow,

    input_manager: InputManager,
    fps_manager: FPSManager,

    models: HashMap<String, Model>,
    ui: HashMap<String, UI>,
    shaders: HashMap<String, Shader>,
    active_shader: String,
    cameras: HashMap<String, Camera3D>,
    active_camera: String,
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

            input_manager: InputManager::new(events, glfw),
            fps_manager: FPSManager::new(),

            ui: HashMap::new(),
            models: HashMap::new(),
            shaders: HashMap::new(),
            active_shader: String::new(),
            cameras: HashMap::new(),
            active_camera: String::new(),
        }
    }

    pub fn add_model(&mut self, name: &str, model: Model) -> &mut Model {
        self.models.insert(name.to_string(), model);
        self.models.get_mut(name).unwrap()
    }
    pub fn get_model(&self, name: &str) -> &Model {
        self.models.get(name).unwrap()
    }

    pub fn add_ui(&mut self, name: &str, ui: UI) -> &mut UI {
        self.ui.insert(name.to_string(), ui);
        self.ui.get_mut(name).unwrap()
    }

    pub fn add_shader(&mut self, name: &str, shader: Shader) -> &mut Shader {
        self.shaders.insert(name.to_string(), shader);

        if self.active_shader.is_empty() {
            self.active_shader = name.to_string();
        }
        self.shaders.get_mut(name).unwrap()
    }

    pub fn add_camera(&mut self, name: &str, camera: Camera3D) -> &mut Camera3D {
        self.cameras.insert(name.to_string(), camera);

        if self.active_camera.is_empty() {
            self.active_camera = name.to_string();
        }

        self.cameras.get_mut(name).unwrap()
    }

    pub fn begin(&mut self) {
        for model in self.models.values_mut() {
            model.ready();
        }

        for camera in self.cameras.values_mut() {
            camera.ready();
        }

        if self.active_camera.is_empty() {
            eprintln!("Warning: No camera found in the scene");
        }

        if self.active_shader.is_empty() {
            eprintln!("Warning: No shader found in the scene");
        }

        //render loop
        self.render_loop();
    }

    /// Call this function recursively to render the scene
    fn render_loop(&mut self) {
        while !self.window.should_close() {
            self.fps_manager.update(|fps| {
                self.window.set_title(&format!("FPS: {}", fps));
            });
            self.input_manager.update();

            for ui in self.ui.values_mut() {
                ui.update(&self.fps_manager, &mut self.input_manager);
            }

            for model in self.models.values_mut() {
                model.behavior(&self.fps_manager, &self.input_manager);
            }

            for camera in self.cameras.values_mut() {
                camera.behavior(&self.fps_manager, &self.input_manager);
            }

            for model in self.models.values_mut() {
                if let Some(shader) = self.shaders.get_mut(&self.active_shader) {
                    model.draw(shader, &self.cameras[&self.active_camera]);
                }
            }

            for ui in self.ui.values_mut() {
                ui.render();
            }

            self.window.swap_buffers();
        }
    }
}
