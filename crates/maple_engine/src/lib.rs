pub mod components;
pub mod context;
pub mod nodes;
pub mod platform;
pub mod resources;
pub mod scene;
pub mod utils;

pub use context::GameContext;
pub use scene::{Scene, SceneBuilder};

pub use nodes::{Buildable, Builder, Node};

pub mod prelude {
    pub use crate::components::*;

    pub use crate::resources::*;

    pub use crate::context::*;

    pub use crate::nodes::*;

    pub use crate::scene::*;

    pub use crate::utils::{Color, Debug};
}
