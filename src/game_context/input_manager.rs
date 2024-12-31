//! The `input_manager` module provides a struct for managing user input, including key presses, mouse buttons, and mouse position.
//!
//! ## Features
//! - `event-driven`: Uses the `glfw` crate to poll events from the window.
//! - `key-presses`: Tracks which keys are currently pressed and which were just pressed.
//! - `mouse-buttons`: Tracks which mouse buttons are currently pressed and which were just pressed.
//!
//! ## Usage
//! Use this within nodes behavior to have dynamic behavior based on user input.
//!
//! ## Example
//! ```rust,ignore
//! impl Behavior for CustomNode {
//!     fn behavior(&mut self, context: &mut GameContext) {
//!         // print all the keys that were just pressed
//!         for key in context.input.key_just_pressed.iter() {
//!            println!("Key just pressed: {:?}", key);
//!        }
//! }
//! ```

use egui_backend::glfw;
use egui_gl_glfw as egui_backend;
use glfw::{GlfwReceiver, Key, MouseButton};
use nalgebra_glm as glm; // Importing the nalgebra_glm crate for mathematical operations
use std::collections::HashSet;

/// Manages the input from the user
pub struct InputManager {
    glfw: glfw::Glfw,
    event_receiver: GlfwReceiver<(f64, glfw::WindowEvent)>,
    /// Stores the events for the current frame
    pub events: Vec<(f64, glfw::WindowEvent)>,
    /// Stores the keys that are currently pressed
    pub keys: HashSet<Key>,
    /// Stores the keys that were just pressed this frame
    pub key_just_pressed: HashSet<Key>,
    /// Stores the mouse buttons that are currently pressed
    pub mouse_buttons: HashSet<MouseButton>,
    /// Stores the mouse buttons that were just pressed this frame
    pub mouse_button_just_pressed: HashSet<MouseButton>,
    /// Stores the current mouse position
    pub mouse_position: glm::Vec2,
    /// Stores the mouse position in the last frameq
    pub last_mouse_position: glm::Vec2,
    /// Stores the change in mouse position since the last frame
    pub mouse_delta: glm::Vec2,
}

impl InputManager {
    /// Creates a new input manager
    pub fn new(events: GlfwReceiver<(f64, glfw::WindowEvent)>, glfw: glfw::Glfw) -> InputManager {
        InputManager {
            glfw,
            event_receiver: events,
            events: Vec::new(), //initialize with a default event
            keys: HashSet::new(),
            key_just_pressed: HashSet::new(),
            mouse_buttons: HashSet::new(),
            mouse_button_just_pressed: HashSet::new(),
            mouse_position: glm::vec2(0.0, 0.0),
            last_mouse_position: glm::vec2(0.0, 0.0),
            mouse_delta: glm::vec2(0.0, 0.0),
        }
    }

    /// update the input data every frame. should be called once per frame before using the input data
    pub fn update(&mut self) {
        self.glfw.poll_events();

        self.mouse_delta = self.mouse_position - self.last_mouse_position;
        self.last_mouse_position = self.mouse_position;

        self.key_just_pressed.clear(); //clear previous frame's keys
        self.mouse_button_just_pressed.clear(); //clear previous frame's mouse buttons

        self.events.clear(); //clear previous frame's events
        self.events = glfw::flush_messages(&self.event_receiver).collect();

        for (_, event) in self.events.iter() {
            match event {
                glfw::WindowEvent::Key(key, _, action, _) => {
                    if *action == glfw::Action::Press {
                        self.keys.insert(*key);
                        self.key_just_pressed.insert(*key); //add the key to the just pressed set
                    } else if *action == glfw::Action::Release {
                        self.keys.remove(key);
                    }
                }

                glfw::WindowEvent::MouseButton(button, action, _) => {
                    if *action == glfw::Action::Press {
                        self.mouse_buttons.insert(*button);
                        self.mouse_button_just_pressed.insert(*button); //add the button to the just pressed set
                    } else if *action == glfw::Action::Release {
                        self.mouse_buttons.remove(button);
                    }
                }
                glfw::WindowEvent::CursorPos(x, y) => {
                    self.mouse_position = glm::vec2(*x as f32, *y as f32);
                    //println!("Mouse position: {:?}", self.mouse_position);
                }
                _ => {}
            }
        }
    }
}
