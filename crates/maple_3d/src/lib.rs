pub mod components;
pub mod gltf;
pub mod math;
pub mod nodes;
pub mod plugin;
pub mod render_passes;

pub mod prelude {
    pub use crate::nodes::{
        camera::{Camera3D, Camera3DBuilder},
        directional_light::{DirectionalLight, DirectionalLightBuilder},
        environment::{Environment, ResolutionScale},
        mesh::{Mesh3D, Mesh3DBuilder},
        point_light::{PointLight, PointLightBuilder},
    };

    pub use crate::gltf::GltfScene;

    pub use crate::components::material::*;
    pub use crate::plugin::Core3D;
}
