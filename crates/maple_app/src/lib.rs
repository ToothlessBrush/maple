pub mod app;
pub mod app_error;
pub mod config;
pub mod default_plugin;
pub mod plugin;

pub use app::*;
pub use plugin::Plugin;

pub mod prelude {
    pub use crate::app::{Init, Running};
    pub use crate::config::*;
    pub use crate::*;
}
