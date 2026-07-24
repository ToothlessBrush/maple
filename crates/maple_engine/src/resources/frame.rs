//! fps_manager use to manage the frame per second of the game and includes statistics about the
//! games frames
//!
//! stuff like delta_time (dt) can be accessed through [`crate::components::EventCtx`] rather then needing to fetch
//! this resource

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

use std::collections::VecDeque;
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

pub struct FrameStats {
    history: VecDeque<f32>,
    max_samples: usize,

    dirty: bool,
    cached_avg_fps: f32,
    sorted_times: Vec<f32>, // slowest-first, cached; percentiles read from this
}

impl FrameStats {
    pub fn new(max_samples: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_samples),
            max_samples,
            dirty: true,
            cached_avg_fps: 0.0,
            sorted_times: Vec::new(),
        }
    }

    pub fn record(&mut self, delta_seconds: f32) {
        self.history.push_back(delta_seconds);
        if self.history.len() > self.max_samples {
            self.history.pop_front();
        }
        self.dirty = true;
    }

    fn ensure_fresh(&mut self) {
        if !self.dirty || self.history.is_empty() {
            return;
        }

        let total_time: f32 = self.history.iter().sum();
        self.cached_avg_fps = self.history.len() as f32 / total_time;

        self.sorted_times.clear();
        self.sorted_times.extend(self.history.iter().copied());
        self.sorted_times.sort_by(|a, b| b.partial_cmp(a).unwrap()); // slowest first

        self.dirty = false;
    }

    pub fn avg_fps(&mut self) -> f32 {
        self.ensure_fresh();
        self.cached_avg_fps
    }

    /// pct as a fraction, e.g. 0.01 for 1% low, 0.05 for 5% low, 0.001 for 0.1% low
    pub fn low_percent(&mut self, pct: f32) -> f32 {
        self.ensure_fresh();
        if self.sorted_times.is_empty() {
            return 0.0;
        }

        let n = ((self.sorted_times.len() as f32) * pct).ceil() as usize;
        let n = n.max(1).min(self.sorted_times.len());

        let slice = &self.sorted_times[0..n];
        let avg_time: f32 = slice.iter().sum::<f32>() / n as f32;
        1.0 / avg_time
    }
}

/// Manages the frame per second of the game
pub struct Frame {
    frame_count: u32,
    /// the time when the game started
    pub start_time: Instant,
    pub elapsed: Duration,

    /// the frames per second updated every second
    pub fps: f32,

    pub stats: FrameStats,

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
            stats: FrameStats::new(1000),
            start_time: Instant::now(),
            elapsed: Duration::default(),
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

        self.elapsed = self.start_time.elapsed();

        // update time delta
        self.time_delta = now.duration_since(self.last_frame_time);
        self.time_delta_f32 = self.time_delta.as_secs_f32();

        self.stats.record(self.time_delta_f32);

        // accumulate time for fixed timestep
        self.fixed_timestep.accumulator += self.time_delta_f32;

        let max_accumulator = self.fixed_timestep.fixed_dt * 5.0;
        self.fixed_timestep.accumulator = self.fixed_timestep.accumulator.min(max_accumulator);

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

    pub fn avg_fps(&mut self) -> f32 {
        self.stats.avg_fps()
    }

    pub fn low_percent(&mut self, percent: f32) -> f32 {
        self.stats.low_percent(percent)
    }
}
