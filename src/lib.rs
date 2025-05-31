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
use nodes::DirectionalLight;
use render_passes::cube_shadow_pass::CubeShadowPass;
use render_passes::{main_pass::MainPass, shadow_pass::ShadowPass};
pub use utils::config::EngineConfig;

use crate::nodes::{Camera3D, Model, PointLight, UI};
use nodes::Node;
use nodes::node::Drawable;
use renderer::Renderer;
use renderer::shader::Shader;

pub mod components;
pub mod context;
pub mod nodes;
pub mod render_passes;
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
    renderer: Renderer,
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
    /// use maple::Engine;
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

        let renderer = Renderer::init();

        Engine {
            context: GameContext::new(events, glfw, window),
            //shadow_map: None,
            config,

            renderer,
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
    /// use maple::Engine;
    /// let mut engine = Engine::init("My Game", 800, 600);
    ///
    /// //set up the scene
    ///
    /// engine.begin();
    /// ```
    pub fn begin(&mut self) -> Result<(), Box<dyn Error>> {
        self.renderer.add_pass(ShadowPass);
        self.renderer.add_pass(CubeShadowPass);
        self.renderer.add_pass(MainPass);

        self.context.emit(Event::Ready);

        if self.context.scene.active_shader.is_empty() {
            eprintln!("INFO: No shader override using default shader");
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

            self.renderer.set_clear_color(self.config.clear_color);
            println!("set_clear: {:?}", now.elapsed().as_secs_f32());

            let now = std::time::Instant::now();
            Renderer::clear();
            println!("clear: {:?}", now.elapsed().as_secs_f32());
            let now = std::time::Instant::now();
            self.renderer.render(&self.context);
            println!("rendering: {:?}", now.elapsed().as_secs_f32());
            let now = std::time::Instant::now();

            self.render_ui_pass();

            println!("ui pass: {:?}", now.elapsed().as_secs_f32());
            let now = std::time::Instant::now();

            // update ecs while rendering
            self.update_context();

            println!("context: {:?}", now.elapsed().as_secs_f32());
            let now = std::time::Instant::now();

            self.update_ui();

            println!("ui: {:?}", now.elapsed().as_secs_f32());
            let now = std::time::Instant::now();

            self.context.emit(Event::Update);

            println!("update: {:?}", now.elapsed().as_secs_f32());
            let now = std::time::Instant::now();

            // swap buffers
            self.context.window.swap_buffers();
            println!("swap buffer: {:?}", now.elapsed().as_secs_f32());
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

/// traverses the scene and returns the nodes of a given type
fn collect_items<N, T>(node: &mut dyn Node, items: &mut Vec<T>)
where
    T: From<&'static mut N>,
    N: 'static,
{
    // Check if the current node matches the target type `N`
    if let Some(target) = node.as_any_mut().downcast_mut::<N>() {
        // Use `unsafe` to extend the lifetime as static (assuming safe usage)
        items.push(T::from(unsafe { &mut *(target as *mut _) }));
    }

    // Recursively collect items from children
    for child in node.get_children_mut().get_all_mut().values_mut() {
        let child_node: &mut dyn Node = &mut **child;
        collect_items::<N, T>(child_node, items);
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

/// draws a given node if it is a model
fn draw_node(node: &mut dyn Node, shader_ptr: *mut Shader, camera_ptr: *mut Camera3D) {
    if let Some(model) = node.as_any_mut().downcast_mut::<Model>() {
        unsafe {
            model.draw(&mut *shader_ptr, &*camera_ptr);
        }
    }

    for child in node.get_children_mut() {
        draw_node(&mut **child.1, shader_ptr, camera_ptr);
    }
}

/// we store the active camera path so in order to get it we need to traverse it
fn traverse_camera_path(
    context: &mut GameContext,
    camera_path: Vec<String>,
) -> Option<&mut Camera3D> {
    // Early return if path is empty
    if camera_path.is_empty() {
        return None;
    }

    let mut current_node = context.scene.get_dyn_mut(&camera_path[0])?;

    for index in &camera_path[1..] {
        current_node = current_node.get_children_mut().get_dyn_mut(index)?;
    }

    current_node.as_any_mut().downcast_mut::<Camera3D>()
}
