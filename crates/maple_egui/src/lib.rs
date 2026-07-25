//! egui implementation for maple

pub mod input;
pub mod plugin;
pub mod render;

pub use egui;

/// common types that are used within this crate
pub mod prelude {
    pub use crate::plugin::EguiPlugin;
    pub use crate::plugin::EguiUpdate;
    pub use egui;
}
