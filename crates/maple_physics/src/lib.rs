//! [`rapier3d`] implemented for maple engine
//!
//! provides nodes such as [`nodes::RigidBody3D`] and [`nodes::Collider3D`] for adding physics
//! behaviors to scene nodes

pub mod nodes;
pub mod plugin;
pub mod resource;

pub use rapier3d::prelude::{ActiveEvents, Group, InteractionGroups, InteractionTestMode};

pub mod prelude {
    pub use crate::ActiveEvents;
    pub use crate::nodes::*;
    pub use crate::plugin::Physics3D;
    pub use crate::resource::{ColliderEnter, ColliderExit, Physics};
}
