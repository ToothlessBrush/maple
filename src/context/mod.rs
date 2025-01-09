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
}
