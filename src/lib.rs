#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
use std::error::Error;
use std::time::Instant;

use components::Event;
pub use nalgebra_glm as glm; // Importing the nalgebra_glm crate for mathematical operations

//re-exporting the engine module
pub use egui_gl_glfw::egui;
pub use egui_gl_glfw::glfw;

use egui_gl_glfw::glfw::Context;
use renderer::shader;

use crate::nodes::{Camera3D, DirectionalLight, Model, PointLight, UI};
use context::scene::{Drawable, Node, Scene};
use renderer::shader::Shader;
use renderer::Renderer;

use components::NodeTransform;

pub mod components;
pub mod context;
pub mod nodes;
pub mod renderer;
pub mod utils;

use context::GameContext;

/// Represents the main game engine.
///
/// The Enigne is responsible for managing the game loop and rendering the scene.
pub struct Engine {
    /// The game context such as the frame, input, nodes, and shaders.
    pub context: GameContext,
    // /// The shadow map used for rendering shadows.
    //pub shadow_map: Option<renderer::shadow_map::ShadowMap>,
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
        glfw.window_hint(glfw::WindowHint::RefreshRate(Some(60)));

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

        // glfw.set_swap_interval(glfw::SwapInterval::None);

        Renderer::init();

        Engine {
            context: GameContext::new(events, glfw, window),
            //shadow_map: None,
        }
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
    pub fn begin(&mut self) -> Result<(), Box<dyn Error>> {
        self.context.emit(Event::Ready);

        if self.context.scene.active_shader.is_empty() {
            eprintln!("Warning: No shader found in the scene");
            self.context.scene.add_shader("default", Shader::default());
        }

        self.update_ui();

        //render loop
        self.render_loop()
    }

    /// The main render loop.
    /// This function is responsible for rendering the scene and updating the game context.
    /// It is called by the `begin` function.
    fn render_loop(&mut self) -> Result<(), Box<dyn Error>> {
        while !self.context.window.should_close() {
            Renderer::clear();

            // queue draw calls
            self.cube_shadow_depth_pass();

            self.render_main_pass();

            self.render_ui_pass();

            // update ecs while rendering
            self.update_context();

            self.update_ui();

            self.context.emit(Event::Update);

            // swap buffers
            self.context.window.swap_buffers();
        }
        Ok(())
    }

    pub fn set_window_title(&mut self, title: &str) {
        self.context.window.set_title(title);
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

    fn update_context(&mut self) {
        let context = &mut self.context;
        context.frame.update();
        context.input.update();
    }

    fn update_ui(&mut self) {
        let nodes = self.context.scene.get_iter::<UI>();

        //map nodes to raw pointer to borrowed twice
        let nodes: Vec<*mut UI> = nodes.map(|node| node as *const UI as *mut UI).collect();

        for ui in nodes {
            unsafe {
                (*ui).update(&mut self.context);
            }
        }
    }

    fn update_nodes(&mut self) {
        self.context.emit(Event::Update);
    }

    fn cube_shadow_depth_pass(&mut self) {
        let context = &mut self.context;
        // let lights: Vec<*mut PointLight> = context
        //     .nodes
        //     .get_iter::<PointLight>()
        //     .map(|light| light as *const PointLight as *mut PointLight)
        //     .collect();

        let lights: &mut Vec<(*mut PointLight, NodeTransform)> = &mut Vec::new();
        for node in context.scene.get_all_mut().values_mut() {
            collect_items::<PointLight, *mut PointLight>(
                &mut **node,
                lights,
                NodeTransform::default(),
            );
        }

        //println!("{:?}", lights);

        for (i, (light, transform)) in lights.iter().enumerate() {
            unsafe {
                // SAFETY: we are using raw pointers here because we guarantee
                // that the nodes vector will not be modified (no adding/removing nodes)
                // during this iteration instead that is needs to be handled through a queue system
                let nodes = context.scene.get_all_mut();

                // println!("{:?}, {:?}", light, transform);

                let nodes = nodes.values_mut().collect::<Vec<&mut Box<dyn Node>>>();

                // let map = std::mem::take(&mut self.context.shadowCubeMaps);

                //println!("{:?}", nodes);

                // Render shadow map
                (**light).render_shadow_map(nodes, *transform, &mut context.shadow_cube_maps, i);

                // Bind uniforms
                let active_shader = context.scene.active_shader.clone();
                if let Some(shader) = context.scene.shaders.get_mut(&active_shader) {
                    (**light).bind_uniforms(shader, i);
                    shader.set_uniform("pointLightLength", (i + 1) as i32);
                }
            }
        }

        let active_shader = context.scene.active_shader.clone();
        if let Some(shader) = context.scene.shaders.get_mut(&active_shader) {
            context
                .shadow_cube_maps
                .bind_shadow_map(shader, "shadowCubeMaps", 2);
        }

        //reset viewport
        Renderer::viewport(
            self.context.window.get_framebuffer_size().0,
            self.context.window.get_framebuffer_size().1,
        );
    }

    fn render_main_pass(&mut self) {
        let context = &mut self.context;

        let active_shader = context.scene.active_shader.clone();

        // // collect all the models
        // let nodes: &mut Vec<*mut Model> = &mut Vec::new();
        // for node in context.scene.get_all_mut().values_mut() {
        //     collect_items::<Model, *mut Model>(&mut **node, nodes);
        // }

        let camera_path = context.active_camera_path.clone();
        let camera = traverse_camera_path(context, camera_path);

        // if let Some(camera) = camera {
        //     // sort models by distance to camera so that they are drawn in the correct order
        //     nodes.sort_by(|a, b| {
        //         let a_distance: f32;
        //         let b_distance: f32;
        //         unsafe {
        //             a_distance = glm::distance2(
        //                 (**a).transform.get_position(),
        //                 &camera.get_position(),
        //             ); // Using squared distance for efficiency
        //             b_distance = glm::distance2(
        //                 (**b).transform.get_position(),
        //                 &camera.get_position(),
        //             ); // Using squared distance for efficiency
        //         }
        //         b_distance
        //             .partial_cmp(&a_distance)
        //             .unwrap_or(std::cmp::Ordering::Equal)
        //     });
        // }

        // Draw the model
        // we use raw pointers here because taking ownership means we need to allocate memory which takes longer and in realtime rendering every ns counts
        if let Some((camera, parent_transform)) = camera {
            let camera_ptr = camera as *const Camera3D as *mut Camera3D;
            let shader_ptr = context
                .scene
                .shaders
                .get_mut(&active_shader)
                .map(|s| &mut **s as *mut Shader);

            if let Some(shader_ptr) = shader_ptr {
                for node in self.context.scene.get_all_mut() {
                    draw_node(
                        &mut **node.1,
                        NodeTransform::default(),
                        shader_ptr,
                        (camera_ptr, parent_transform),
                    );
                }
            }
        }
    }

    fn render_ui_pass(&mut self) {
        let nodes = self.context.scene.get_iter::<UI>();

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
}

/// Collects all the models in the scene for rendering.
// fn collect_models<T>(node: &mut dyn Node, models: &mut Vec<T>)
// where
//     T: From<&'static mut Model>,
// {
//     // Check if the current node is a Model
//     if let Some(model) = node.as_any_mut().downcast_mut::<Model>() {
//         models.push(T::from(unsafe { &mut *(model as *mut _) }));
//     }

//     // Recursively collect models from children
//     for child in node.get_children_mut().get_all_mut().values_mut() {
//         let child_node: &mut dyn Node = &mut **child;
//         collect_models(child_node, models);
//     }
// }

// fn collect_lights<T>(
//     node: &mut dyn Node,
//     lights: &mut Vec<(T, NodeTransform)>,
//     parent_transform: NodeTransform,
// ) where
//     T: From<&'static mut PointLight>,
// {
//     let world_transform = parent_transform + *node.get_transform();
//     if let Some(light) = node.as_any_mut().downcast_mut::<PointLight>() {
//         lights.push((T::from(unsafe { &mut *(light as *mut _) }), world_transform));
//     }

//     for child in node.get_children_mut().get_all_mut().values_mut() {
//         let child_node: &mut dyn Node = &mut **child;
//         collect_lights(child_node, lights, world_transform);
//     }
// }

fn collect_items<N, T>(
    node: &mut dyn Node,
    items: &mut Vec<(T, NodeTransform)>,
    parent_transform: NodeTransform,
) where
    T: From<&'static mut N>,
    N: 'static,
{
    let world_transform = parent_transform + *node.get_transform();
    // Check if the current node matches the target type `N`
    if let Some(target) = node.as_any_mut().downcast_mut::<N>() {
        // Use `unsafe` to extend the lifetime as static (assuming safe usage)
        items.push((
            T::from(unsafe { &mut *(target as *mut _) }),
            world_transform,
        ));
    }

    // Recursively collect items from children
    for child in node.get_children_mut().get_all_mut().values_mut() {
        let child_node: &mut dyn Node = &mut **child;
        collect_items::<N, T>(child_node, items, world_transform);
    }
}

/// Converts a mutable reference to a Model to a raw pointer.
impl From<&'static mut Model> for *mut Model {
    fn from(model: &'static mut Model) -> Self {
        model as *mut Model
    }
}

impl From<&'static mut PointLight> for *mut PointLight {
    fn from(light: &'static mut PointLight) -> Self {
        light as *mut PointLight
    }
}

fn draw_node(
    node: &mut dyn Node,
    parent_transform: NodeTransform,
    shader_ptr: *mut Shader,
    camera_ptr: (*mut Camera3D, NodeTransform),
) {
    let world_transform = parent_transform + *node.get_transform();

    if let Some(model) = node.as_any_mut().downcast_mut::<Model>() {
        unsafe {
            model.draw(
                &mut *shader_ptr,
                (&*(camera_ptr.0), camera_ptr.1),
                world_transform,
            );
        }
    }

    for child in node.get_children_mut() {
        draw_node(&mut **child.1, world_transform, shader_ptr, camera_ptr);
    }
}

fn traverse_camera_path(
    context: &mut GameContext,
    camera_path: Vec<String>,
) -> Option<(&mut Camera3D, NodeTransform)> {
    // Early return if path is empty
    if camera_path.is_empty() {
        return None;
    }

    let mut current_node = context.scene.get_dyn_mut(&camera_path[0])?;
    let mut current_transform = NodeTransform::default();

    for index in &camera_path[1..] {
        current_transform = current_transform + *current_node.get_transform();
        current_node = current_node.get_children_mut().get_dyn_mut(&index)?;
    }

    if let Some(camera) = current_node.as_any_mut().downcast_mut::<Camera3D>() {
        Some((camera, current_transform))
    } else {
        None
    }
}
