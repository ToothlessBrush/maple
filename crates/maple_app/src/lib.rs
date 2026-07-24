//! the core App of the maple engine
//!
//! contains the base [`App`] struct which manages the game loop the window and plugins,

pub mod app;
pub mod app_error;
pub mod config;
pub(crate) mod default_plugin;
pub mod plugin;

pub use app::*;
pub use plugin::Plugin;

pub mod prelude {
    pub use crate::app::{Init, Running};
    pub use crate::config::*;
    pub use crate::*;
}
