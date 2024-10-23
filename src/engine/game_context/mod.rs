pub mod fps_manager;
pub mod input_manager;
pub mod node_manager;
pub mod nodes;

use egui_backend::glfw;
use egui_gl_glfw as egui_backend;
use glfw::GlfwReceiver;

use fps_manager::FPSManager;
use input_manager::InputManager;
use node_manager::NodeManager;

use std::cell::RefCell;

pub struct GameContext {
    pub nodes: RefCell<NodeManager>,
    pub frame: RefCell<FPSManager>,
    pub input: RefCell<InputManager>,
}

impl GameContext {
    pub fn new(events: GlfwReceiver<(f64, glfw::WindowEvent)>, glfw: glfw::Glfw) -> GameContext {
        GameContext {
            nodes: RefCell::new(NodeManager::new()),
            frame: RefCell::new(FPSManager::new()),
            input: RefCell::new(InputManager::new(events, glfw)),
        }
    }
}
