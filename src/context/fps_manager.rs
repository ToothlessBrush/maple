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

/// times a callback and stores it in target
pub fn time_callback<F, R>(target: &mut f32, func: F) -> R
where
    F: FnOnce() -> R,
{
    let start = std::time::Instant::now();
    let result = func();
    *target = start.elapsed().as_secs_f32();
    result
}

/// times a code snippet
#[macro_export]
macro_rules! time {
    ($target:expr, $body:block) => {
        $crate::context::fps_manager::time_callback($target, || $body)
    };
}

/// per function timings
#[derive(Default)]
pub struct FrameInfo {
    /// time to clear the frame
    pub clear_time: f32,
    /// time to renderr the frame
    pub render_time: f32,
    /// time to render the ui
    pub ui_pass_time: f32,
    /// time to update context
    pub context_update_time: f32,
    /// time to update ui
    pub ui_update_time: f32,
    ///time to emit events
    pub event_emit_time: f32,
    /// time to swap buffers
    pub swap_buffers_time: f32,
    /// time the total frame took
    pub total_frame_time: f32,
}

use std::fmt;

impl fmt::Display for FrameInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Frame Timings:")?;
        writeln!(
            f,
            "  Clear Time           : {:>12}",
            format_duration(self.clear_time)
        )?;
        writeln!(
            f,
            "  Render Time          : {:>12}",
            format_duration(self.render_time)
        )?;
        writeln!(
            f,
            "  UI Pass Time         : {:>12}",
            format_duration(self.ui_pass_time)
        )?;
        writeln!(
            f,
            "  Context Update Time  : {:>12}",
            format_duration(self.context_update_time)
        )?;
        writeln!(
            f,
            "  UI Update Time       : {:>12}",
            format_duration(self.ui_update_time)
        )?;
        writeln!(
            f,
            "  Event Emit Time      : {:>12}",
            format_duration(self.event_emit_time)
        )?;
        writeln!(
            f,
            "  Swap Buffers Time    : {:>12}",
            format_duration(self.swap_buffers_time)
        )?;
        writeln!(
            f,
            "  Total Frame Time     : {:>12}",
            format_duration(self.total_frame_time)
        )
    }
}

fn format_duration(seconds: f32) -> String {
    if seconds >= 1.0 {
        format!("{:.5} s", seconds)
    } else if seconds >= 0.001 {
        format!("{:.3} ms", seconds * 1000.0)
    } else {
        format!("{:.0} ns", (seconds * 1_000_000_000.0))
    }
}

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
    /// delta time in seconds as a float
    pub time_delta_f32: f32,
    /// info on the frame
    pub frame_info: FrameInfo,
}

impl Default for FPSManager {
    /// Creates a new FPSManager with default values
    fn default() -> Self {
        Self::new()
    }
}

impl FPSManager {
    /// Creates a new FPSManager
    pub(crate) fn new() -> Self {
        FPSManager {
            frame_count: 0,
            fps: 0,
            start_time: Instant::now(),
            last_frame_time: Instant::now(),
            last_update_time: Instant::now(),
            time_delta: Duration::default(),
            time_delta_f32: 0.0,
            frame_info: FrameInfo::default(),
        }
    }

    /// Updates the FPSManager should be called once per frame.
    pub(crate) fn update(&mut self) {
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
