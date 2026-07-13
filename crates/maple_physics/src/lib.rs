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
