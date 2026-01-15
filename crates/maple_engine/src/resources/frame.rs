//! fps_manager use to manage the frame per second of the game
//!
//! ## Usage
//! the most common use is to grab the time_delta to create consistant movement regardless of framerate
//!
//! ## Example
//! ```rust
//! use maple::{
//!     nodes::{Buildable, Builder, Empty},
//!     math,
//!     components::Event,
//! };
//!
//! Empty::builder()
//!     .on(Event::Update, |node, ctx| {
//!         node.transform.position += math::vec3(0.0, 0.0, 10.0 * ctx.frame.time_delta_f32)
//!     })
//!     .build();
//!
//! ```

/// times a callback and stores it in target
#[allow(dead_code)]
pub fn time_callback<F, R>(target: &mut f32, func: F) -> R
where
    F: FnOnce() -> R,
{
    let start = std::time::Instant::now();
    let result = func();
    *target = start.elapsed().as_secs_f32();
    result
}

#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};

#[cfg(target_arch = "wasm32")]
use web_time::{Duration, Instant};

use crate::context::Resource;

pub struct FixedTimeStep {
    accumulator: f32,
    fixed_dt: f32,
}

impl FixedTimeStep {
    pub fn new(tps: u32) -> Self {
        Self {
            accumulator: 0.0,
            fixed_dt: 1.0 / tps as f32,
        }
    }
}

/// Manages the frame per second of the game
pub struct Frame {
    frame_count: u32,
    /// the time when the game started
    pub start_time: Instant,

    /// the frames per second updated every second
    pub fps: f32,
    last_frame_time: Instant,
    /// the time between the last frame and the current frame
    pub time_delta: Duration,
    /// delta time in seconds as a float
    pub time_delta_f32: f32,
    /// fixed timestep for fixed update events
    pub fixed_timestep: FixedTimeStep,
}

impl Resource for Frame {}

impl Default for Frame {
    /// Creates a new Frame with default values
    fn default() -> Self {
        Self::new()
    }
}

impl Frame {
    /// Creates a new Frame
    pub(crate) fn new() -> Self {
        Frame {
            frame_count: 0,
            fps: 0.0,
            start_time: Instant::now(),
            last_frame_time: Instant::now(),
            time_delta: Duration::default(),
            time_delta_f32: 0.0,
            fixed_timestep: FixedTimeStep::new(60),
        }
    }

    /// Updates the Frame should be called once per frame.
    pub(crate) fn update(&mut self) {
        self.frame_count += 1;
        let now = Instant::now();

        // update time delta
        self.time_delta = now.duration_since(self.last_frame_time);
        self.time_delta_f32 = self.time_delta.as_secs_f32();

        // accumulate time for fixed timestep
        self.fixed_timestep.accumulator += self.time_delta_f32;

        self.fps = 1.0 / self.time_delta_f32;
        self.last_frame_time = now;
    }

    /// Checks if a fixed update should run and consumes the accumulator
    ///
    /// Returns true if the accumulator has enough time for a fixed update step.
    /// Should be called in a while loop to handle multiple fixed updates per frame.
    pub fn should_fixed_update(&mut self) -> bool {
        if self.fixed_timestep.accumulator >= self.fixed_timestep.fixed_dt {
            self.fixed_timestep.accumulator -= self.fixed_timestep.fixed_dt;
            true
        } else {
            false
        }
    }

    /// Returns the fixed delta time (1/60 of a second by default)
    pub fn fixed_delta_time(&self) -> f32 {
        self.fixed_timestep.fixed_dt
    }
}
