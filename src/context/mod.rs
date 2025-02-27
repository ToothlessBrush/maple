//! This module contains the game context, which contains all the necessary information for the game to run.
//!
//! This includes the window, the nodes, the frame manager, the input manager, and the shadow distance.

use crate::components::Event;
use fps_manager::*;
use input_manager::*;
use scene::Scene;

pub mod fps_manager;
pub mod input_manager;
pub mod scene;

use egui_backend::glfw;
use egui_gl_glfw as egui_backend;
use glfw::GlfwReceiver;

//use crate::renderer::depth_map_array::DepthMapArray;

use crate::{
    components::NodeTransform,
    nodes::Camera3D,
    renderer::{depth_cube_map_array::DepthCubeMapArray, shader::Shader},
};
use std::cell::RefCell;

use scene::Node;

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

    pub shadow_cube_maps: DepthCubeMapArray,
    //pub shadow_maps: DepthMapArray,
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
            // shadow_maps: DepthMapArray::gen_map(
            //     1024,
            //     1024,
            //     10,
            //     Shader::from_slice(
            //         include_str!("../../res/shaders/depthShader/depthShader.vert"),
            //         include_str!("../../res/shaders/depthShader/depthShader.frag"),
            //         None,
            //     ),
            // ),
            active_camera_path: Vec::new(),
            shadow_cube_maps: DepthCubeMapArray::gen_map(
                1024,
                1024,
                10,
                Shader::from_slice(
                    include_str!("../../res/shaders/cubeDepthShader/cubeDepthShader.vert"),
                    include_str!("../../res/shaders/cubeDepthShader/cubeDepthShader.frag"),
                    Some(include_str!(
                        "../../res/shaders/cubeDepthShader/cubeDepthShader.geom"
                    )),
                ),
            ),
        }
    }

    /// emits an event to the currently loaded nodes in the context
    ///
    /// # Arguments
    /// - `event` - which event to emit
    ///
    /// # example
    /// ```rust
    /// engine.context.emit(Custom("damage".to_string()));
    /// ```
    pub fn emit(&mut self, event: Event) {
        let nodes = &mut self.scene as *mut Scene;

        // dont delete nodes to avoid hanging pointer
        unsafe { (*nodes).emit(event, self) }
    }

    /// lock the cursor inside the window.
    ///
    /// # Arguments
    /// - `lock` - Whether to lock the cursor or not.
    pub fn lock_cursor(&mut self, lock: bool) {
        if lock {
            self.window.set_cursor_mode(glfw::CursorMode::Disabled);
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
