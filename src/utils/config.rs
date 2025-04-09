//! the EngineCofig is used to create engine settings before the context is initialized
//!
//! some things need to be known by the engine beforehand such as the window title or Resolution
//!
//! # Example
//! ```rust
//! use quaturn::Engine;
//!
//! let engine = Engine::new(EngineConfig::default());
//! ```

use crate::math;

use super::{color, Color};

/// the mode refers to if its fullscreenm borderless or windowed.
pub enum WindowMode {
    /// Fullscreen Mode
    FullScreen,
    /// Borderless Fullscreen Mode
    Borderless,
    /// Windowed Mode (recommended for development)
    Windowed,
}

/// the resolution of the window in pixels
pub struct Resolution {
    /// the width **1080**x1920
    pub width: u32,
    /// the height 1080x**1920**
    pub height: u32,
}

/// the config of the engine
pub struct EngineConfig {
    /// the title of the window
    pub window_title: String,
    /// mode of the window such as FullScreen, or Windowed
    pub window_mode: WindowMode,
    /// resolution of the window.
    ///
    /// see [Resolution]
    pub resolution: Resolution,

    /// background Color
    ///
    /// the color that the screen is cleared with before rendering the next frame
    pub clear_color: math::Vec4,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            window_title: "".to_string(),
            window_mode: WindowMode::Windowed,
            resolution: Resolution {
                width: 1920,
                height: 1080,
            },
            clear_color: color::GREY.into(),
        }
    }
}
