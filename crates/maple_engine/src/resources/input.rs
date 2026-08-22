//! The `input_manager` module provides a struct for managing user input, including key presses, mouse buttons, and mouse position.
//!
//! ## Features
//! - `event-driven`: Uses the `glfw` crate to poll events from the window.
//! - `key-presses`: Tracks which keys are currently pressed and which were just pressed.
//! - `mouse-buttons`: Tracks which mouse buttons are currently pressed and which were just pressed.
//!
//! ## Usage
//! Use this within nodes behavior to have dynamic behavior based on user input.

use glam::{self as math, Vec2};
use std::collections::HashSet;
use winit::{
    event::{DeviceEvent, ElementState, MouseScrollDelta, WindowEvent},
    keyboard::PhysicalKey,
}; // Importing the nalgebra_glm crate for mathematical operations

pub use winit::event::MouseButton;
pub use winit::event::TouchPhase;
pub use winit::keyboard::KeyCode;

use crate::context::Resource;

impl Resource for Input {}

/// Manages the input from the user
pub struct Input {
    events: Vec<WindowEvent>,

    pub keys: HashSet<KeyCode>,
    pub key_just_pressed: HashSet<KeyCode>,
    pub key_just_released: HashSet<KeyCode>,

    pub mouse_buttons: HashSet<MouseButton>,
    pub mouse_button_just_pressed: HashSet<MouseButton>,
    pub mouse_button_just_released: HashSet<MouseButton>,

    pub cursor_position: math::Vec2,
    pub mouse_delta: math::Vec2,
    pub cursor_entered: bool,
    pub cursor_exit: bool,

    pub text_input: String,

    pub scroll_delta_lines: math::Vec2,
    pub scroll_delta_pixels: math::Vec2,
    pub scroll_phase: Option<TouchPhase>,
}

impl Input {
    /// Creates a new input manager with a window reference
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            keys: HashSet::new(),
            key_just_pressed: HashSet::new(),
            key_just_released: HashSet::new(),
            mouse_buttons: HashSet::new(),
            mouse_button_just_pressed: HashSet::new(),
            mouse_button_just_released: HashSet::new(),
            cursor_position: math::vec2(0.0, 0.0),
            mouse_delta: math::vec2(0.0, 0.0),
            cursor_entered: false,
            cursor_exit: false,
            text_input: String::new(),
            scroll_delta_lines: math::vec2(0.0, 0.0),
            scroll_delta_pixels: math::vec2(0.0, 0.0),
            scroll_phase: None,
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
                            self.key_just_released.insert(keycode);
                        }
                    }
                }

                if event.state == ElementState::Pressed {
                    if let Some(text) = &event.text {
                        for c in text.chars().filter(|c| !c.is_control()) {
                            self.text_input.push(c);
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
                    self.mouse_button_just_released.insert(*button);
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                let new_position = math::vec2(position.x as f32, position.y as f32);
                self.cursor_position = new_position;
            }
            WindowEvent::CursorEntered { .. } => {
                self.cursor_entered = true;
            }
            WindowEvent::CursorLeft { .. } => {
                self.cursor_exit = true;
            }
            WindowEvent::MouseWheel { delta, phase, .. } => {
                self.scroll_phase = Some(*phase);
                match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        self.scroll_delta_lines += math::vec2(*x, *y);
                    }
                    MouseScrollDelta::PixelDelta(pos) => {
                        self.scroll_delta_pixels += math::vec2(pos.x as f32, pos.y as f32)
                    }
                }
            }
            _ => {}
        }
    }

    pub fn end_frame(&mut self) {
        self.key_just_pressed.clear();
        self.key_just_released.clear();
        self.mouse_button_just_pressed.clear();
        self.mouse_button_just_released.clear();

        self.mouse_delta = math::vec2(0.0, 0.0);
        self.cursor_entered = false;
        self.cursor_exit = false;
        self.text_input.clear();
        self.scroll_delta_lines = Vec2::ZERO;
        self.scroll_delta_pixels = Vec2::ZERO;

        self.events.clear();
    }

    /// get a vector by specifying 4 inputs which map to the 4 directions
    pub fn get_vector(
        &self,
        neg_x: &KeyCode,
        pos_x: &KeyCode,
        neg_y: &KeyCode,
        pos_y: &KeyCode,
    ) -> Vec2 {
        let mut out = Vec2::default();
        if self.keys.contains(pos_x) {
            out += Vec2::X;
        }
        if self.keys.contains(neg_x) {
            out += Vec2::NEG_X;
        }
        if self.keys.contains(pos_y) {
            out += Vec2::Y;
        }
        if self.keys.contains(neg_y) {
            out += Vec2::NEG_Y;
        }
        out
    }

    /// Cursor position converted to logical points (physical / ppp).
    pub fn cursor_position_points(&self, scale_factor: f32) -> math::Vec2 {
        self.cursor_position / scale_factor
    }
}
