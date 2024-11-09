use egui_backend::glfw;
use egui_gl_glfw as egui_backend;
use egui_gl_glfw::glfw::Context;

use game_context::nodes::camera::Camera3D;
use game_context::nodes::model::{self, Model};
use game_context::nodes::ui::UI;
use renderer::shader::Shader;
use renderer::Renderer;

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
        for model in self.context.nodes.models.values_mut() {
            model.ready();
        }

        for camera in self.context.nodes.cameras.values_mut() {
            camera.ready();
        }

        if self.context.nodes.active_camera.is_empty() {
            eprintln!("Warning: No camera found in the scene");
        }

        if self.context.nodes.active_shader.is_empty() {
            eprintln!("Warning: No shader found in the scene");
        }

        self.shadow_map = Some(renderer::shadow_map::ShadowMap::gen_map(
            2048,
            2048,
            Shader::new("res/shaders/depthShader"),
        ));

        //render loop
        self.render_loop();
    }

    fn render_loop(&mut self) {
        while !self.context.window.should_close() {
            Renderer::clear();

            let light_direction = glm::normalize(&glm::vec3(1.0, 1.0, 1.0));
            let light_projections = glm::ortho(-10.0, 10.0, -10.0, 10.0, 0.1, 75.0);
            let light_position = light_direction * 20.0;
            let light_view = glm::look_at(
                &light_position,
                &glm::vec3(0.0, 0.0, 0.0),
                &glm::vec3(0.0, 1.0, 0.0),
            );
            let light_space_matrix = light_projections * light_view;

            //render from lights orthographic view to shadow map buffer with shadow map shaders
            self.shadow_map.as_mut().unwrap().render_shadow_map(
                // Render shadow map
                &mut |depth_shader: &mut Shader| {
                    // Draw models
                    {
                        depth_shader.bind();
                        depth_shader.set_uniform_mat4f("u_lightSpaceMatrix;", &light_space_matrix);
                        for model in self.context.nodes.models.values_mut() {
                            model.draw_shadow(depth_shader, &light_space_matrix);
                        }
                        depth_shader.unbind();
                    }
                },
            );

            self.shadow_map.as_mut().unwrap().bind_shadow_map(
                self.context
                    .nodes
                    .shaders
                    .get_mut(&self.context.nodes.active_shader)
                    .unwrap(),
                "shadowMap",
                2,
            ); //bind shadow map texture

            //bind light space matrix to active shader
            self.context
                .nodes
                .shaders
                .get_mut(&self.context.nodes.active_shader)
                .unwrap()
                .bind();
            self.context
                .nodes
                .shaders
                .get_mut(&self.context.nodes.active_shader)
                .unwrap()
                .set_uniform_mat4f("u_lightProjection", &light_space_matrix);

            Renderer::viewport(
                self.context.window.get_framebuffer_size().0,
                self.context.window.get_framebuffer_size().1,
            );

            // Update frame and input
            {
                let context = &mut self.context;
                context.frame.update(|fps| {
                    context.window.set_title(&format!("FPS: {}", fps));
                });
                context.input.update();
            }

            //render shadow map

            //note if a node is removed while in these scope it can cause a dangling pointer

            // Update UIs
            {
                let nodes: Vec<*mut UI> = self
                    .context
                    .nodes
                    .uis
                    .values_mut()
                    .map(|ui| ui as *mut UI)
                    .collect();
                for ui in nodes {
                    unsafe {
                        (*ui).update(&mut self.context);
                    }
                }
            }

            // Update models
            {
                let nodes: Vec<*mut Model> = self
                    .context
                    .nodes
                    .models
                    .values_mut()
                    .map(|m| m as *mut Model)
                    .collect();
                for model in nodes {
                    unsafe {
                        (*model).behavior(&mut self.context);
                    }
                }
            }

            // Update cameras
            {
                let nodes: Vec<*mut Camera3D> = self
                    .context
                    .nodes
                    .cameras
                    .values_mut()
                    .map(|d| d as *mut Camera3D)
                    .collect();
                for camera in nodes {
                    unsafe {
                        (*camera).behavior(&mut self.context);
                    }
                }
            }

            // Draw models
            {
                let context = &mut self.context;
                for (_, model) in context.nodes.models.iter_mut() {
                    if let Some(shader) =
                        context.nodes.shaders.get_mut(&context.nodes.active_shader)
                    {
                        model.draw(shader, &context.nodes.cameras[&context.nodes.active_camera]);
                    }
                }
            }

            // Render UIs
            {
                let nodes: Vec<*mut UI> = self
                    .context
                    .nodes
                    .uis
                    .values_mut()
                    .map(|ui| ui as *mut UI)
                    .collect();
                for ui in nodes {
                    unsafe {
                        (*ui).render(&mut self.context);
                    }
                }
            }

            self.context.window.swap_buffers();
        }
    }
}
