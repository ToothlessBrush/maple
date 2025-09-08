pub mod components;
pub mod context;
pub mod input;
pub mod nodes;
pub mod scene;
pub mod utils;

pub use context::GameContext;
pub use glam as math;
pub use scene::{Scene, SceneBuilder};

pub use nodes::{Buildable, Builder, Node};

pub mod prelude {
    pub use crate::components::{
        NodeTransform,
        event_reciever::{Event, EventReceiver},
    };

    pub use crate::input::*;

    pub use crate::context::*;

    pub use crate::nodes::*;

    pub use crate::scene::*;

    pub use crate::utils::{Color, Debug};

    pub use glam as math;
    pub use math::{Mat4, Quat, Vec2, Vec3, Vec4};
}
