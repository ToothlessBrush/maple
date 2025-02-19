//! This module contains the game context, which contains all the necessary information for the game to run.
//!
//! This includes the window, the nodes, the frame manager, the input manager, and the shadow distance.

use fps_manager::*;
use input_manager::*;
use node_manager::NodeManager;

pub mod fps_manager;
pub mod input_manager;
pub mod node_manager;

use egui_backend::glfw;
use egui_gl_glfw as egui_backend;
use glfw::GlfwReceiver;

use crate::{
    components::NodeTransform,
    nodes::Camera3D,
    renderer::{depth_cube_map_array::DepthCubeMapArray, shader::Shader},
};
use std::cell::RefCell;

use node_manager::Node;

// use fps_manager::FPSManager;
// use input_manager::InputManager;
// use node_manager::NodeManager;

/// The main game context, containing all the necessary information for the game to run.
/// This includes the window, the nodes, the frame manager, the input manager, and the shadow distance.
pub struct GameContext {
    /// The window of the game.
    pub window: glfw::PWindow,
    /// The node manager of the game.
    pub nodes: NodeManager,
    /// The frame manager of the game.
    pub frame: FPSManager,
    /// The input manager of the game.
    pub input: InputManager,
    /// The shadow distance of the game.
    pub shadow_distance: f32,
    /// path to the active camera
    pub active_camera_path: Vec<String>,

    pub shadowCubeMaps: DepthCubeMapArray,
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

            nodes: NodeManager::new(),
            frame: FPSManager::new(),
            input: InputManager::new(events, glfw),
            shadow_distance: 100.0,
            active_camera_path: Vec::new(),
            shadowCubeMaps: DepthCubeMapArray::gen_map(
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

    pub fn set_main_camera(&mut self, camera: *const Camera3D) {
        let mut search_path = Vec::<String>::new();

        // Iterate through the nodes and try to find the camera path.
        for node in &mut self.nodes {
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
