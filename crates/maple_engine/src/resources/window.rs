use std::sync::Arc;

use glam::Vec2;

use crate::context::Resource;

pub struct Window {
    window: Arc<winit::window::Window>,
    cursor_locked: bool,
    cursor_lock_applied: bool,
}

impl Resource for Window {}

impl Window {
    pub fn new(window: Arc<winit::window::Window>) -> Self {
        let mut window_manager = Self {
            window: window.clone(),
            cursor_locked: false,
            cursor_lock_applied: false,
        };
        window_manager.apply_cursor_lock();
        window_manager
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
    pub fn set_cursor_locked(&mut self, locked: bool) {
        if self.cursor_locked != locked {
            self.cursor_locked = locked;
            self.apply_cursor_lock(); // Apply the change immediately
        }
    }

    pub fn is_cursor_locked(&self) -> bool {
        self.cursor_locked
    }

    pub fn screen_size_pixels(&self) -> Vec2 {
        let size = self.window.inner_size();
        glam::vec2(size.width as f32, size.height as f32)
    }

    /// Window's scale factor / pixels-per-point (DPI), e.g. 1.0, 1.5, 2.0
    pub fn scale_factor(&self) -> f32 {
        self.window.scale_factor() as f32
    }

    /// Logical (points) screen size = physical pixels / scale factor.
    pub fn screen_size_points(&self) -> Vec2 {
        self.screen_size_pixels() / self.scale_factor()
    }
}
