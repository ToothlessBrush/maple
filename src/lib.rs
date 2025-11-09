pub use glam as math;
#[cfg(feature = "3d")]
pub use maple_3d;
pub use maple_app as app;
pub use maple_derive as derive;
pub use maple_engine as engine;
pub use maple_renderer as renderer;

pub mod prelude {
    pub use crate::app::prelude::*;
    pub use crate::derive::Node;
    pub use crate::engine::prelude::*;

    #[cfg(feature = "3d")]
    pub use crate::maple_3d;

    // dont export renderer prelude since renderer isnt used as often

    /// re-export glam as math
    pub use glam as math;
    pub use math::{Mat4, Quat, Vec2, Vec3, Vec4};
}
