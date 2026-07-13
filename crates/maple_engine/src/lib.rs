pub mod asset;
pub mod color;
pub mod components;
pub mod context;
pub mod nodes;
pub mod platform;
pub mod resources;
pub mod scene;

pub use context::GameContext;
pub use scene::{Scene, SceneBuilder};

pub use nodes::{Buildable, Builder, Node};

pub mod prelude {
    pub use crate::components::*;

    pub use crate::resources::*;

    pub use crate::context::*;

    pub use crate::nodes::*;

    pub use crate::scene::*;

    pub use crate::asset::{AssetHandle, AssetLibrary};

    pub use crate::color::Color;
}
