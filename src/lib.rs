#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
use std::error::Error;

// I wish I used glow ngl
#[allow(warnings)]
pub(crate) mod gl {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use components::Event;
use context::scene::Scene;
use egui_gl_glfw::glfw::Cursor;
use egui_gl_glfw::glfw::WindowMode;
pub use nalgebra_glm as math;

//re-exporting the engine module
pub use egui_gl_glfw::egui;
pub use egui_gl_glfw::glfw;

use egui_gl_glfw::glfw::Context;
use nodes::directional_light::DirectionalLightBufferData;
use nodes::point_light::PointLightBufferData;
use nodes::DirectionalLight;
use utils::config::EngineConfig;

use crate::nodes::{Camera3D, Model, PointLight, UI};
use nodes::node::Drawable;
use nodes::Node;
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
    /// configuration of the engine
    pub config: EngineConfig,
    /// renderer of the engine
    _renderer: Renderer,
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
    /// let mut engine = Engine::init(EngineConfig {
    ///     ..Default::default()
    /// });
    /// ```
    pub fn init(config: EngineConfig) -> Engine {
        use glfw::fail_on_errors;
        let mut glfw = glfw::init(fail_on_errors!()).unwrap();
        glfw.window_hint(glfw::WindowHint::ContextVersion(4, 6));
        glfw.window_hint(glfw::WindowHint::OpenGlProfile(
            glfw::OpenGlProfileHint::Core,
        ));
        glfw.window_hint(glfw::WindowHint::DoubleBuffer(true));
        glfw.window_hint(glfw::WindowHint::Resizable(false));
        glfw.window_hint(glfw::WindowHint::Samples(Some(SAMPLES)));
        //glfw.window_hint(glfw::WindowHint::RefreshRate(Some(60)));

        let (mut window, events) = match config.window_mode {
            utils::config::WindowMode::Windowed => glfw
                .create_window(
                    config.resolution.width,
                    config.resolution.height,
                    &config.window_title,
                    WindowMode::Windowed,
                )
                .expect("failed to create window"),
            utils::config::WindowMode::FullScreen => glfw.with_primary_monitor(|g, monitor| {
                let mut width = config.resolution.width;
                let mut height = config.resolution.height;

                if let Some(monitor) = &monitor {
                    if let Some(vid_mode) = monitor.get_video_mode() {
                        width = vid_mode.width;
                        height = vid_mode.height;
                    }
                }

                g.create_window(
                    width,
                    height,
                    &config.window_title,
                    monitor.map_or(WindowMode::Windowed, |m| WindowMode::FullScreen(m)),
                )
                .expect("failed to create window")
            }),
            _ => glfw
                .create_window(
                    config.resolution.width,
                    config.resolution.height,
                    &config.window_title,
                    WindowMode::Windowed,
                )
                .expect("failed to create window"),
        };

        //set up input polling
        window.set_key_polling(true);
        window.set_cursor_pos_polling(true);
        window.set_mouse_button_polling(true);
        window.set_scroll_polling(true);
        window.make_current();

        window.set_cursor(Some(Cursor::standard(glfw::StandardCursor::IBeam)));

        if glfw.supports_raw_motion() {
            window.set_raw_mouse_motion(true);
        }

        //load grahpics api
        Renderer::context(&mut window);

        glfw.set_swap_interval(glfw::SwapInterval::None);

        Renderer::init();

        Engine {
            context: GameContext::new(events, glfw, window),
            //shadow_map: None,
            config,

            _renderer: Renderer::default(),
        }
    }
    /// load a scene into the games Context
    ///
    /// # Arguments
    /// - `scene`: the scene to be added to the context's Scene
    pub fn load_scene(&mut self, scene: Scene) {
        self.context.scene.load(scene);
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
            self.context
                .scene
                .add_shader("default", Shader::use_default());
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
            let now = std::time::Instant::now();
            let total = now;
            Renderer::set_clear_color(self.config.clear_color);

            Renderer::clear();

            self.shadow_depth_pass();

            //println!("shadow: {:?}", now.elapsed().as_secs_f32());
            let now = std::time::Instant::now();

            // queue draw calls
            self.cube_shadow_depth_pass();

            //println!("cube_shadow: {:?}", now.elapsed().as_secs_f32());
            let now = std::time::Instant::now();

            self.render_main_pass();

            //println!("main pass: {:?}", now.elapsed().as_secs_f32());
            let now = std::time::Instant::now();

            self.render_ui_pass();

            //println!("cube_shadow: {:?}", now.elapsed().as_secs_f32());
            let now = std::time::Instant::now();

            // update ecs while rendering
            self.update_context();

            //println!("context: {:?}", now.elapsed().as_secs_f32());
            let now = std::time::Instant::now();

            self.update_ui();

            //println!("ui: {:?}", now.elapsed().as_secs_f32());
            let now = std::time::Instant::now();

            self.context.emit(Event::Update);

            //println!("update: {:?}", now.elapsed().as_secs_f32());
            let now = std::time::Instant::now();

            // swap buffers
            self.context.window.swap_buffers();
            //println!("swap buffer: {:?}", now.elapsed().as_secs_f32());
            use colored::*;
            let elapsed_time = total.elapsed().as_secs_f32();
            if elapsed_time > 0.01 {
                println!(
                    "{}",
                    format!("Total time: {:.3} seconds", elapsed_time).red()
                );
            } else {
                println!("Total time: {:.3} seconds", elapsed_time);
            }
        }
        Ok(())
    }
    /// sets the window set_title
    ///
    /// # Arguements
    /// - 'title' - the title
    pub fn set_window_title(&mut self, title: &str) {
        self.context.window.set_title(title);
    }

    fn update_context(&mut self) {
        let context = &mut self.context;
        let now = std::time::Instant::now();

        context.frame.update();

        println!("frame: {:?}", now.elapsed().as_secs_f32());
        let now = std::time::Instant::now();

        context.input.update();

        println!("input: {:?}", now.elapsed().as_secs_f32());
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

    fn shadow_depth_pass(&mut self) {
        let context = &mut self.context;

        let lights: &mut Vec<(*mut DirectionalLight, NodeTransform)> = &mut Vec::new();
        for node in context.scene.get_all_mut().values_mut() {
            collect_items::<DirectionalLight, *mut DirectionalLight>(
                &mut **node,
                lights,
                NodeTransform::default(),
            );
        }

        if let Some(camera) = traverse_camera_path(context, context.active_camera_path.clone()) {
            let (camera, transform) = camera;

            let camera_transform = camera.transform;

            let mut offset = 0;

            let mut buffer_data = Vec::<DirectionalLightBufferData>::new();
            let mut size = 0;

            context.shadow_maps.bind_framebuffer();

            for (i, (light, _node_transform)) in lights.iter().enumerate() {
                let nodes = context.scene.get_all_mut();

                let nodes = nodes.values_mut().collect::<Vec<&mut Box<dyn Node>>>();

                // lights dont draw themselves so its safe to derefrence to get the other nodes
                unsafe {
                    context
                        .shadow_maps
                        .commit_layer(offset as u32, (**light).num_cascades as i32);

                    (**light).render_shadow_map(
                        nodes,
                        &mut context.shadow_maps,
                        offset,
                        &(transform + camera_transform),
                    );

                    offset += (**light).num_cascades;
                }

                renderer::depth_map_array::DepthMapArray::unbind_framebuffer();

                // Bind uniforms
                let active_shader = context.scene.active_shader.clone();
                if let Some(shader) = context.scene.shaders.get_mut(&active_shader) {
                    unsafe {
                        buffer_data.push((**light).get_buffered_data());
                        size += 1;
                    }
                }
            }

            //bind to buffer
            context
                .direct_light_buffer
                .set_data(size, buffer_data.as_slice());

            //bind texture to its texture slot
            let active_shader = context.scene.active_shader.clone();
            if let Some(shader) = context.scene.shaders.get_mut(&active_shader) {
                context.shadow_maps.bind_shadow_map(shader, "shadowMaps", 5);
                context.direct_light_buffer.bind(0);
            }
        }
    }

    fn cube_shadow_depth_pass(&mut self) {
        let context = &mut self.context;

        let lights: &mut Vec<(*mut PointLight, NodeTransform)> = &mut Vec::new();
        for node in context.scene.get_all_mut().values_mut() {
            collect_items::<PointLight, *mut PointLight>(
                &mut **node,
                lights,
                NodeTransform::default(),
            );
        }

        context.shadow_cube_maps.bind_framebuffer();

        let mut size = 0;
        let mut buffer_data = Vec::<PointLightBufferData>::new();

        for (i, (light, transform)) in lights.iter().enumerate() {
            unsafe {
                context.shadow_cube_maps.commit_layer(i as u32);

                // SAFETY: we are using raw pointers here because we guarantee
                // that the nodes vector will not be modified (no adding/removing nodes)
                // during this iteration instead that is needs to be handled through a queue system
                let nodes = context.scene.get_all_mut();

                let nodes = nodes.values_mut().collect::<Vec<&mut Box<dyn Node>>>();

                // Render shadow map
                (**light).render_shadow_map(nodes, *transform, &mut context.shadow_cube_maps, i);

                // Bind uniforms
                let active_shader = context.scene.active_shader.clone();
                if let Some(shader) = context.scene.shaders.get_mut(&active_shader) {
                    buffer_data.push((**light).get_buffered_data());
                    size += 1;
                }
            }
        }

        context.shadow_cube_maps.unbind_framebuffer();

        context
            .point_light_buffer
            .set_data(size, buffer_data.as_slice());

        //bind texture
        let active_shader = context.scene.active_shader.clone();
        if let Some(shader) = context.scene.shaders.get_mut(&active_shader) {
            shader.bind();
            shader.set_uniform("scene.biasFactor", context.scene_state.bias_factor);
            shader.set_uniform("scene.biasOffset", context.scene_state.bias_offset);
            shader.set_uniform("scene.ambient", context.scene_state.ambient_light);

            context
                .shadow_cube_maps
                .bind_shadow_map(shader, "shadowCubeMaps", 2);
            context.point_light_buffer.bind(1);
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
        //             a_distance = math::distance2(
        //                 (**a).transform.get_position(),
        //                 &camera.get_position(),
        //             ); // Using squared distance for efficiency
        //             b_distance = math::distance2(
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

// /// Collects all the models in the scene for rendering.
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

impl From<&'static mut DirectionalLight> for *mut DirectionalLight {
    fn from(value: &'static mut DirectionalLight) -> Self {
        value as *mut DirectionalLight
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
        current_node = current_node.get_children_mut().get_dyn_mut(index)?;
    }

    current_node
        .as_any_mut()
        .downcast_mut::<Camera3D>()
        .map(|camera| (camera, current_transform))
}
