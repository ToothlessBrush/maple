#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
pub use nalgebra_glm as glm; // Importing the nalgebra_glm crate for mathematical operations

//re-exporting the engine module
pub use egui_gl_glfw::egui;
pub use egui_gl_glfw::glfw;

use egui_gl_glfw::glfw::Context;

use game_context::node_manager::{Drawable, Node, NodeManager};
use game_context::nodes::camera::Camera3D;
use game_context::nodes::directional_light::DirectionalLight;
use game_context::nodes::model::Model;
use game_context::nodes::ui::UI;
use renderer::shader::Shader;
use renderer::Renderer;

pub mod game_context;
pub mod renderer;

use game_context::GameContext;

/// Represents the main game engine.
///
/// The Enigne is responsible for managing the game loop and rendering the scene.
pub struct Engine {
    /// The game context such as the frame, input, nodes, and shaders.
    pub context: GameContext,
    /// The shadow map used for rendering shadows.
    pub shadow_map: Option<renderer::shadow_map::ShadowMap>,
}

/// The number of samples for anti-aliasing.
const SAMPLES: u32 = 8;

impl Engine {
    /// Initializes the game engine.
    ///
    /// # Arguments
    /// - `window_title`: The title of the window.
    /// - `window_width`: The width of the window.
    /// - `window_height`: The height of the window.
    ///
    /// # Returns
    /// A new instance of the Engine.
    ///
    /// # Example
    /// ```rust
    /// use quaturn::Engine;
    /// let mut engine = Engine::init("My Game", 800, 600);
    /// ```
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

    /// sets the clear color of the window.
    ///
    /// the renderer clears the screen before rendering the next frame with the color set here.
    /// # Arguments
    /// - `r`: The red value of the color.
    /// - `g`: The green value of the color.
    /// - `b`: The blue value of the color.
    /// - `a`: The alpha value of the color.
    ///
    /// # Example
    /// ```rust
    /// use quaturn::Engine;
    /// let mut engine = Engine::init("My Game", 800, 600);
    /// engine.set_clear_color(0.1, 0.1, 0.1, 1.0);
    /// ```
    pub fn set_clear_color(&self, r: f32, g: f32, b: f32, a: f32) {
        Renderer::set_clear_color([r, g, b, a]);
    }

    /// starts the gamme/render loop.
    ///
    /// this function is responsible for rendering the scene and updating the game context.
    ///
    /// # Example
    /// ```rust
    /// use quaturn::Engine;
    /// let mut engine = Engine::init("My Game", 800, 600);
    ///
    /// //set up the scene
    ///
    /// engine.begin();
    /// ```
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

    /// The main render loop.
    /// This function is responsible for rendering the scene and updating the game context.
    /// It is called by the `begin` function.
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
                        let mut nodes: &mut Vec<&mut Model> = &mut Vec::new();
                        for node in context.nodes.get_all_mut().values_mut() {
                            collect_models(&mut **node, &mut nodes);
                        }

                        // Render shadow map
                        (*light).render_shadow_map(nodes);

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

                // collect all the models
                let mut nodes: &mut Vec<*mut Model> = &mut Vec::new();
                for node in context.nodes.get_all_mut().values_mut() {
                    collect_models(&mut **node, &mut nodes);
                }

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
                                (**model).draw(&mut *shader_ptr, &*camera_ptr);
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
            //std::thread::sleep(std::time::Duration::from_millis(10)); //sleep for 1ms
        }
    }
}

/// Collects all the models in the scene for rendering.
fn collect_models<T>(node: &mut dyn Node, models: &mut Vec<T>)
where
    T: From<&'static mut Model>,
{
    // Check if the current node is a Model
    if let Some(model) = node.as_any_mut().downcast_mut::<Model>() {
        models.push(T::from(unsafe { &mut *(model as *mut _) }));
    }

    // Recursively collect models from children
    for child in node.get_children().get_all_mut().values_mut() {
        let child_node: &mut dyn Node = &mut **child;
        collect_models(child_node, models);
    }
}

/// Converts a mutable reference to a Model to a raw pointer.
impl From<&'static mut Model> for *mut Model {
    fn from(model: &'static mut Model) -> Self {
        model as *mut Model
    }
}
