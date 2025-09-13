pub mod app;
pub mod app_error;
pub mod config;
pub mod plugin;

pub use app::App;
pub use plugin::Plugin;

pub mod prelude {
    pub use crate::App;
    pub use crate::Plugin;
    pub use crate::config::*;
}
