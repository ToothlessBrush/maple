//! This module contains the game context, which contains all the necessary information for the game to run.
//!
//! This includes the window, the nodes, the frame manager, the input manager, and the shadow distance.

use crate::Event;
use crate::nodes::Node;
use fps_manager::*;
use input_manager::*;
use scene::Scene;

pub mod fps_manager;
pub mod input_manager;
pub mod scene;

use egui_backend::glfw;
use egui_gl_glfw as egui_backend;
use glfw::GlfwReceiver;

use crate::nodes::Camera3D;

// use fps_manager::FPSManager;
// use input_manager::InputManager;
// use node_manager::Scene;

/// The main game context, containing all the necessary information for the game to run.
/// This includes the window, the nodes, the frame manager, the input manager, and the shadow distance.
pub struct GameContext {
    /// The window of the game.
    pub window: glfw::PWindow,
    /// The node manager of the game.
    pub scene: Scene,
    /// The frame manager of the game.
    pub frame: FPSManager,
    /// The input manager of the game.
    pub input: InputManager,
    /// The shadow distance of the game.
    pub shadow_distance: f32,
    /// path to the active camera
    pub active_camera_path: Vec<String>,
    // TODO: move these to the renderer and store the renderer on the engine (since we dont want
    // users to modify this)
}

impl GameContext {
    /// Creates a new game context with the given events, glfw, and window.
    ///
    /// # Arguments
    /// - `events` - The input events of the game.
    /// - `glfw` - The glfw context of the game.
    /// - `window` - The window of the game.
    ///
    /// # Returns
    /// The new game context.
    pub fn new(
        events: GlfwReceiver<(f64, glfw::WindowEvent)>,
        glfw: glfw::Glfw,
        window: glfw::PWindow,
    ) -> GameContext {
        GameContext {
            window,

            scene: Scene::new(),
            frame: FPSManager::new(),
            input: InputManager::new(events, glfw),
            shadow_distance: 100.0,
            active_camera_path: Vec::new(),
        }
    }

    /// emits an event to the currently loaded nodes in the context
    ///
    /// # Arguments
    /// - `event` - which event to emit
    ///
    /// # example
    /// ```rust
    /// engine = maple::Engine::init(EngineConfig::default());
    ///
    /// engine.context.emit(Custom("damage".to_string()));
    /// ```
    pub fn emit(&mut self, event: Event) {
        let nodes = &mut self.scene as *mut Scene;

        // we need to pass self when we are borrowing self.nodes and idk another solution

        unsafe { (*nodes).emit(event, self) }
    }

    /// lock the cursor inside the window.
    ///
    /// # Arguments
    /// - `lock` - Whether to lock the cursor or not.
    pub fn lock_cursor(&mut self, lock: bool) {
        if lock {
            self.window.set_cursor_mode(glfw::CursorMode::Disabled);
            self.input.reset_mouse_delta();
        } else {
            self.window.set_cursor_mode(glfw::CursorMode::Normal);
        }
    }

    pub fn get_cursor_mode(&self) -> glfw::CursorMode {
        self.window.get_cursor_mode()
    }

    pub fn set_main_camera(&mut self, camera: *const Camera3D) {
        let mut search_path = Vec::<String>::new();

        // Iterate through the nodes and try to find the camera path.
        for node in &mut self.scene {
            if let Some(path) = Self::traverse_nodes(node, Vec::new(), camera) {
                search_path = path;
                break; // Exit once the camera is found
            }
        }

        if search_path.is_empty() {
            println!("no matching result");
        } else {
            println!("camera found at path: {:?}", search_path);
            self.active_camera_path = search_path;
        }
    }

    /// time since last frame. this is really useful if you want smooth movement
    ///
    /// by multiplying somthing that is frame dependant such as a transform it will move at a
    /// consistant speed even if the frame rate is different
    ///
    /// # example
    /// ```rust
    /// use maple::nodes::{Nodebuilder, Empty, EmptyBuilder};
    ///
    /// NodeBuilder::<Empty>::create()
    ///     .on(Event::Update, |node, ctx| {
    ///         node.transform.rotate_euler_xyz(math::vec3(0.0, 90.0 * ctx.time_delta(), 0.0));
    ///     })
    /// ```
    pub fn time_delta(&self) -> f32 {
        self.frame.time_delta_f32
    }

    fn traverse_nodes(
        node: (&String, &mut Box<dyn Node>),
        parent_path: Vec<String>,
        camera: *const Camera3D,
    ) -> Option<Vec<String>> {
        let mut current_path = parent_path.clone();
        current_path.push(node.0.clone());

        // Check if the current node is the camera we're searching for
        if let Some(current_camera) = node.1.as_any().downcast_ref::<Camera3D>() {
            if std::ptr::eq(current_camera, camera) {
                return Some(current_path); // Return the path if camera matches
            }
        }

        // Recursively check each child node
        for child in node.1.get_children_mut() {
            if let Some(path) = Self::traverse_nodes(child, current_path.clone(), camera) {
                return Some(path); // Return path if camera is found in child
            }
        }

        None // Return None if the camera is not found in this node or its children
    }
}
