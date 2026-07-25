//! config used by the renderer

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Config used by the renderer
#[derive(Default, Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct RenderConfig {
    /// vsync mode of the window
    pub vsync: VsyncMode,
}

/// Vertical sync on or off
#[derive(Default, Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum VsyncMode {
    /// I guess this turns vsync off
    #[default]
    Off,
    /// I guess this turns vsync on
    On,
}
