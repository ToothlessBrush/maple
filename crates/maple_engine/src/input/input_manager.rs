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
use std::{collections::HashSet, sync::Arc};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event::{DeviceEvent, ElementState, WindowEvent},
    keyboard::PhysicalKey,
    window::Window,
}; // Importing the nalgebra_glm crate for mathematical operations

pub use winit::event::MouseButton;
pub use winit::keyboard::KeyCode;

use crate::context::Resource;

impl Resource for InputManager {}

/// Manages the input from the user
pub struct InputManager {
    window: Arc<Window>, // local window so it can call cursor commands
    events: Vec<WindowEvent>,
    pub keys: HashSet<KeyCode>,
    pub key_just_pressed: HashSet<KeyCode>,
    pub mouse_buttons: HashSet<MouseButton>,
    pub mouse_button_just_pressed: HashSet<MouseButton>,
    pub cursor_position: math::Vec2,
    pub mouse_delta: math::Vec2,
    cursor_locked: bool,
    cursor_lock_applied: bool,
}

impl InputManager {
    /// Creates a new input manager with a window reference
    pub fn new(window: Arc<Window>) -> InputManager {
        let mut input_manager = InputManager {
            window: window.clone(),
            events: Vec::new(),
            keys: HashSet::new(),
            key_just_pressed: HashSet::new(),
            mouse_buttons: HashSet::new(),
            mouse_button_just_pressed: HashSet::new(),
            cursor_position: math::vec2(0.0, 0.0),
            mouse_delta: math::vec2(0.0, 0.0),
            cursor_locked: false,
            cursor_lock_applied: false,
        };

        // Apply initial cursor lock state
        input_manager.apply_cursor_lock();
        input_manager
    }

    // Internal method to apply cursor lock state
    fn apply_cursor_lock(&mut self) {
        if self.cursor_locked && !self.cursor_lock_applied {
            // Lock the cursor
            match self
                .window
                .set_cursor_grab(winit::window::CursorGrabMode::Locked)
            {
                Ok(_) => {
                    self.cursor_lock_applied = true;
                    self.window.set_cursor_visible(false);

                    // Don't try to center cursor immediately - let it settle first
                    // The centering will happen in the first few mouse move events
                }
                Err(e) => {
                    eprintln!("Failed to lock cursor: {:?}", e);
                }
            }
        } else if !self.cursor_locked && self.cursor_lock_applied {
            // Unlock the cursor
            match self
                .window
                .set_cursor_grab(winit::window::CursorGrabMode::None)
            {
                Ok(_) => {
                    self.cursor_lock_applied = false;
                    self.window.set_cursor_visible(true);
                }
                Err(e) => {
                    eprintln!("Failed to unlock cursor: {:?}", e);
                }
            }
        }
    }

    pub fn handle_device_event(&mut self, event: &DeviceEvent) {
        #[allow(clippy::single_match)]
        match event {
            DeviceEvent::MouseMotion { delta } => {
                let delta_vec = math::vec2(delta.0 as f32, delta.1 as f32);

                self.mouse_delta += delta_vec;
            }
            _ => {}
        }
    }

    /// Handles a winit input event
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
                self.cursor_position = new_position;
            }
            _ => {}
        }
    }

    pub fn end_frame(&mut self) {
        self.key_just_pressed.clear();
        self.mouse_button_just_pressed.clear();
        self.mouse_delta = math::vec2(0.0, 0.0);

        self.events.clear();
    }

    /// Toggle cursor lock state
    pub fn set_cursor_locked(&mut self, locked: bool) {
        if self.cursor_locked != locked {
            self.cursor_locked = locked;
            self.apply_cursor_lock(); // Apply the change immediately
        }
    }

    pub fn is_cursor_locked(&self) -> bool {
        self.cursor_locked
    }
}
