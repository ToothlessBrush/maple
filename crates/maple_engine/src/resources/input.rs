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
use std::{collections::HashSet, sync::Arc};
use winit::{
    event::{DeviceEvent, ElementState, MouseScrollDelta, WindowEvent},
    keyboard::PhysicalKey,
    window::Window,
}; // Importing the nalgebra_glm crate for mathematical operations

pub use winit::event::MouseButton;
pub use winit::event::TouchPhase;
pub use winit::keyboard::KeyCode;

use crate::context::Resource;

impl Resource for Input {}

/// Manages the input from the user
pub struct Input {
    window: Arc<Window>, // local window so it can call cursor commands
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

    cursor_locked: bool,
    cursor_lock_applied: bool,
}

impl Input {
    /// Creates a new input manager with a window reference
    pub fn new(window: Arc<Window>) -> Self {
        let mut input_manager = Self {
            window: window.clone(),
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
                    log::error!("Failed to lock cursor: {:?}", e);
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
                    log::error!("Failed to unlock cursor: {:?}", e);
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

    pub fn screen_size_pixels(&self) -> math::Vec2 {
        let size = self.window.inner_size();
        math::vec2(size.width as f32, size.height as f32)
    }

    /// Window's scale factor / pixels-per-point (DPI), e.g. 1.0, 1.5, 2.0
    pub fn scale_factor(&self) -> f32 {
        self.window.scale_factor() as f32
    }

    /// Logical (points) screen size = physical pixels / scale factor.
    pub fn screen_size_points(&self) -> math::Vec2 {
        self.screen_size_pixels() / self.scale_factor()
    }

    /// Cursor position converted to logical points (physical / ppp).
    pub fn cursor_position_points(&self) -> math::Vec2 {
        self.cursor_position / self.scale_factor()
    }
}
