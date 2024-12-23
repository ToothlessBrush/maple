use egui_backend::glfw;
use egui_gl_glfw as egui_backend;
use egui_gl_glfw::glfw::Context;

use game_context::node_manager::{Behavior, Drawable, Node, NodeManager, NodeTransform, Ready};
use game_context::nodes::camera::Camera3D;
use game_context::nodes::directional_light::DirectionalLight;
use game_context::nodes::empty::Empty;
use game_context::nodes::model::{self, Model};
use game_context::nodes::ui::UI;
use renderer::shader::Shader;
use renderer::Renderer;

use std::any::Any;

use nalgebra_glm as glm;

pub mod game_context;
pub mod renderer;
pub mod utils;

use game_context::GameContext;

pub struct Engine {
    //pub window: glfw::PWindow,
    pub context: GameContext,
    pub shadow_map: Option<renderer::shadow_map::ShadowMap>,
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
            context: GameContext::new(events, glfw, window),
            shadow_map: None,
        }
    }

    pub fn set_clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        Renderer::set_clear_color([r, g, b, a]);
    }

    pub fn begin(&mut self) {
        self.context.nodes.ready();

        if self.context.nodes.active_camera.is_empty() {
            eprintln!("Warning: No camera found in the scene");
        }

        if self.context.nodes.active_shader.is_empty() {
            eprintln!("Warning: No shader found in the scene");
        }

        self.shadow_map = Some(renderer::shadow_map::ShadowMap::gen_map(
            8192,
            8192,
            Shader::new(
                "res/shaders/depthShader/depthShader.vert",
                "res/shaders/depthShader/depthShader.frag",
                None,
            ),
        ));

        //render loop
        self.render_loop();
    }

    fn render_loop(&mut self) {
        while !self.context.window.should_close() {
            Renderer::clear();

            // Update frame and input
            {
                let context = &mut self.context;
                context.frame.update(|fps| {
                    context.window.set_title(&format!("FPS: {}", fps));
                });
                context.input.update();
            }

            // Render shadow map
            {
                let context = &mut self.context;
                let lights: Vec<*mut DirectionalLight> = context
                    .nodes
                    .get_iter::<DirectionalLight>()
                    .map(|light| light as *const DirectionalLight as *mut DirectionalLight)
                    .collect();

                for light in lights {
                    unsafe {
                        // SAFETY: we are using raw pointers here because we guarantee
                        // that the nodes vector will not be modified (no adding/removing nodes)
                        // during this iteration instead that is needs to be handled through a queue system

                        // Render shadow map
                        (*light).render_shadow_map(&mut context.nodes.get_iter::<Model>());

                        // Bind uniforms
                        let active_shader = context.nodes.active_shader.clone();
                        if let Some(shader) = context.nodes.shaders.get_mut(&active_shader) {
                            (*light).bind_uniforms(shader);
                        }
                    }
                }
            }

            //reset viewport
            Renderer::viewport(
                self.context.window.get_framebuffer_size().0,
                self.context.window.get_framebuffer_size().1,
            );

            //note if a node is removed while in these scope it can cause a dangling pointer

            // Update UIs
            {
                let nodes = self.context.nodes.get_iter::<UI>();

                //map nodes to raw pointer to borrowed twice
                let nodes: Vec<*mut UI> = nodes.map(|node| node as *const UI as *mut UI).collect();

                for ui in nodes {
                    unsafe {
                        (*ui).update(&mut self.context);
                    }
                }
            }

            {
                let nodes = &mut self.context.nodes as *mut NodeManager;
                // SAFETY: we are using raw pointers here because we guarantee
                // that the nodes vector will not be modified (no adding/removing nodes)
                // during this iteration instead that is needs to be handled through a queue system
                unsafe { (*nodes).behavior(&mut self.context) };
            }

            // Draw models
            {
                let context = &mut self.context;

                let active_shader = context.nodes.active_shader.clone();
                let active_camera = context.nodes.active_camera.clone();

                let nodes: Vec<*mut Model> = context
                    .nodes
                    .get_iter::<Model>()
                    .map(|model| model as *const Model as *mut Model)
                    .collect();

                let camera = context.nodes.get::<Camera3D>(&active_camera).map(|c| c);

                if let Some(camera) = camera {
                    let camera_ptr = &*camera as *const Camera3D as *mut Camera3D;
                    let shader_ptr = context
                        .nodes
                        .shaders
                        .get_mut(&active_shader)
                        .map(|s| &mut **s as *mut Shader);

                    if let Some(shader_ptr) = shader_ptr {
                        for model in nodes {
                            unsafe {
                                // SAFETY: we are using raw pointers here because we guarantee
                                // that the nodes vector will not be modified (no adding/removing nodes)
                                // during this iteration instead that is needs to be handled through a queue system
                                (*model).draw(&mut *shader_ptr, &*camera_ptr);
                            }
                        }
                    }
                }
            }

            // Render UIs
            {
                let nodes = self.context.nodes.get_iter::<UI>();

                //map nodes to raw pointer to borrowed twice
                let nodes: Vec<*mut UI> = nodes.map(|node| node as *const UI as *mut UI).collect();

                for ui in nodes {
                    unsafe {
                        // SAFETY: we are using raw pointers here because we guarantee
                        // that the nodes vector will not be modified (no adding/removing nodes)
                        // during this iteration instead that is needs to be handled through a queue system
                        (*ui).render(&mut self.context)
                    }
                }
            }

            self.context.window.swap_buffers();
            std::thread::sleep(std::time::Duration::from_millis(10)); //sleep for 1ms
        }
    }
}
