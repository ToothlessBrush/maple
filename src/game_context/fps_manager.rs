//! fps_manager use to manage the frame per second of the game
//!
//! ## Usage
//! the most common use is to grab the time_delta to make the game run at the same speed on different machines
//!
//! ## Example
//! ```rust
//! impl Behavior for CustomNode {
//!    fn behavior(&mut self, context: &mut GameContext) {
//!       let time_delta = context.fps_manager.time_delta;
//!       self.apply_transform(&mut |t| {
//!          t.transform(glm::vec3(0.0, 0.0, 1.0) * time_delta.as_secs_f32()); // move 1 unit per second
//!      });
//! }
//! ```
//!

use std::time::{Duration, Instant};
//use egui_gl_glfw::glfw;

/// Manages the frame per second of the game
pub struct FPSManager {
    frame_count: u32,
    /// the time when the game started
    pub start_time: Instant,
    last_frame_time: Instant,
    last_update_time: Instant,
    /// the time between the last frame and the current frame
    pub time_delta: Duration,
}

impl Default for FPSManager {
    /// Creates a new FPSManager with default values
    fn default() -> Self {
        Self::new()
    }
}

impl FPSManager {
    /// Creates a new FPSManager
    pub fn new() -> Self {
        FPSManager {
            frame_count: 0,
            start_time: Instant::now(),
            last_frame_time: Instant::now(),
            last_update_time: Instant::now(),
            time_delta: Duration::default(),
        }
    }

    /// Updates the FPSManager should be called once per frame.
    pub fn update<T: FnMut(u32)>(&mut self, mut update_fn: T) {
        self.frame_count += 1;
        let now = Instant::now();
        self.time_delta = now.duration_since(self.last_frame_time);
        if now.duration_since(self.last_update_time) >= Duration::from_secs(1) {
            update_fn(self.frame_count);
            self.frame_count = 0;
            self.last_update_time = now;
        }
        self.last_frame_time = now;
    }
}
