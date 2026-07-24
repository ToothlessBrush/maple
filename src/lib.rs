#[doc = include_str!("../README.md")]
/// math types from [`glam`]
pub use glam as math;

/// 3d rendering
#[cfg(feature = "3d")]
pub use maple_3d;

/// the core App
pub use maple_app as app;

/// spatial audio
#[cfg(feature = "audio")]
pub use maple_audio as audio;

/// derive macros
pub use maple_derive as derive;

/// core engine implementation
pub use maple_engine as engine;

/// physics with [`rapier3d`]
#[cfg(feature = "physics")]
pub use maple_physics as physics;

/// core renderer implementation
pub use maple_renderer as renderer;

/// the prelude exposes almost everything you need to get started
pub mod prelude {
    pub use crate::app::prelude::*;
    pub use crate::derive::Node;
    pub use crate::engine::prelude::*;
    pub use crate::renderer::prelude::*;

    #[cfg(feature = "3d")]
    pub use crate::maple_3d::prelude::*;

    #[cfg(feature = "physics")]
    pub use crate::physics::prelude::*;

    #[cfg(feature = "audio")]
    pub use crate::audio::prelude::*;

    /// re-export glam as math
    use glam as math;
    pub use math::{Mat4, Quat, Vec2, Vec3, Vec4};
}
