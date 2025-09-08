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
//! ```rust
//! use maple::components::Event;
//! use maple::nodes::{Empty, Buildable, Builder};
//! use maple::math;
//! use maple::Key;
//!
//! Empty::builder()
//!     .on(Event::Update, move |node, context| {
//!         // move forward when W is pressed
//!         if context.input.keys.contains(&Key::W) {
//!             node.transform.position += math::vec3(1.0, 0.0, 0.0)
//!         }
//!     })
//!     .build();
//! ```

use glam as math;
use std::collections::HashSet;
use winit::{
    event::{ElementState, MouseButton, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
}; // Importing the nalgebra_glm crate for mathematical operations

/// Manages the input from the user
pub struct InputManager {
    /// Stores the events for the current frame
    pub events: Vec<WindowEvent>,
    /// Stores the keys that are currently pressed
    pub keys: HashSet<KeyCode>,
    /// Stores the keys that were just pressed this frame
    pub key_just_pressed: HashSet<KeyCode>,
    /// Stores the mouse buttons that are currently pressed
    pub mouse_buttons: HashSet<MouseButton>,
    /// Stores the mouse buttons that were just pressed this frame
    pub mouse_button_just_pressed: HashSet<MouseButton>,
    /// Stores the current mouse position
    pub mouse_position: math::Vec2,
    /// Stores the mouse position in the last frameq
    pub last_mouse_position: math::Vec2,
    /// Stores the change in mouse position since the last frame
    pub mouse_delta: math::Vec2,
    /// flag to check if this is the first mouse input (to avoid massive mouse_delta)
    first_mouse: bool,
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InputManager {
    /// Creates a new input manager
    pub fn new() -> InputManager {
        InputManager {
            events: Vec::new(),
            keys: HashSet::new(),
            key_just_pressed: HashSet::new(),
            mouse_buttons: HashSet::new(),
            mouse_button_just_pressed: HashSet::new(),
            mouse_position: math::vec2(0.0, 0.0),
            last_mouse_position: math::vec2(0.0, 0.0),
            mouse_delta: math::vec2(0.0, 0.0),
            first_mouse: true,
        }
    }

    /// handles a winit input event
    pub fn handle_event(&mut self, event: &WindowEvent) {
        self.events.push(event.clone());

        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(keycode) = event.physical_key {
                    match event.state {
                        ElementState::Pressed => {
                            if !self.keys.contains(&keycode) {
                                self.key_just_pressed.insert(keycode);
                            }
                            self.keys.insert(keycode);
                        }
                        ElementState::Released => {
                            self.keys.remove(&keycode);
                        }
                    }
                }
            }

            WindowEvent::MouseInput { state, button, .. } => match state {
                ElementState::Pressed => {
                    if !self.mouse_button_just_pressed.contains(button) {
                        self.mouse_button_just_pressed.insert(*button);
                    }
                    self.mouse_buttons.insert(*button);
                }
                ElementState::Released => {
                    self.mouse_buttons.remove(button);
                }
            },

            WindowEvent::CursorMoved { position, .. } => {
                let new_position = math::vec2(position.x as f32, position.y as f32);

                if self.first_mouse {
                    self.last_mouse_position = new_position;
                    self.mouse_delta = math::vec2(0.0, 0.0);
                    self.first_mouse = false;
                } else {
                    self.mouse_delta = new_position - self.last_mouse_position;
                    self.last_mouse_position = self.mouse_position
                }

                self.mouse_position = new_position;
            }

            _ => {}
        }
    }

    pub fn end_frame(&mut self) {
        self.key_just_pressed.clear();
        self.mouse_button_just_pressed.clear();

        self.events.clear();
    }

    /// reset the mouse position so the offset it 0
    pub fn reset_mouse_delta(&mut self) {
        self.mouse_delta = math::vec2(0.0, 0.0);
        self.last_mouse_position = self.mouse_position;
    }
}
