pub mod assets;
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
        mesh_instance::{MeshInstance3D, MeshInstance3DBuilder},
        point_light::{PointLight, PointLightBuilder},
    };

    pub use crate::assets::materials::pbr_material::PbrMaterial;

    pub use crate::gltf::GltfScene;

    pub use crate::assets::material::{
        AlphaMode, Material, MaterialInstance, MaterialInstanceMut, MaterialInstanceRef,
    };

    pub use crate::assets::mesh::Mesh3D;
    pub use crate::assets::primitives::*;

    pub use crate::plugin::Core3D;
}
