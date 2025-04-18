//! fps_manager use to manage the frame per second of the game
//!
//! ## Usage
//! the most common use is to grab the time_delta to make the game run at the same speed on different machines
//!
//! ## Example
//! ```rust,ignore
//! impl Behavior for CustomNode {
//!    fn behavior(&mut self, context: &mut GameContext) {
//!       let time_delta = context.fps_manager.time_delta;
//!       self.apply_transform(&mut |t| {
//!          t.transform(math::vec3(0.0, 0.0, 1.0) * time_delta.as_secs_f32()); // move 1 unit per second
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

    /// the frames per second updated every second
    pub fps: u32,
    last_frame_time: Instant,
    last_update_time: Instant,
    /// the time between the last frame and the current frame
    pub time_delta: Duration,
    pub time_delta_f32: f32,
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
            fps: 0,
            start_time: Instant::now(),
            last_frame_time: Instant::now(),
            last_update_time: Instant::now(),
            time_delta: Duration::default(),
            time_delta_f32: 0.0,
        }
    }

    /// Updates the FPSManager should be called once per frame.
    pub fn update(&mut self) {
        self.frame_count += 1;
        let now = Instant::now();

        // update time delta
        self.time_delta = now.duration_since(self.last_frame_time);

        self.time_delta_f32 = self.time_delta.as_secs_f32();

        if now.duration_since(self.last_update_time) >= Duration::from_secs(1) {
            self.fps = self.frame_count;
            self.frame_count = 0;
            self.last_update_time = now;
        }
        self.last_frame_time = now;
    }
}
