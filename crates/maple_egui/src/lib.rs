pub mod input;
pub mod plugin;
pub mod render;

pub use egui;

pub mod prelude {
    pub use crate::plugin::EguiPlugin;
    pub use crate::plugin::EguiUpdate;
    pub use egui;
}
