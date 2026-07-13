use glam::Vec3;
use maple_engine::asset::IntoAsset;
use maple_renderer::types::Vertex;

use crate::prelude::Mesh3D;

/// simplest possible mesh shape
///
/// abc counter clockwise order will be perpendicular to normal direction
pub struct Triangle {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
}

impl Default for Triangle {
    fn default() -> Self {
        Self {
            a: Vec3::new(-0.5, 0.0, -0.5),
            b: Vec3::new(0.0, 0.0, 0.5),
            c: Vec3::new(0.5, 0.0, -0.5),
        }
    }
}

impl IntoAsset<Mesh3D> for Triangle {
    fn into_asset(
        self,
        loader: &<Mesh3D as maple_engine::asset::Asset>::Loader,
        _library: &maple_engine::prelude::AssetLibrary,
    ) -> Result<Mesh3D, maple_engine::asset::LoadErr> {
        let normal = (self.c - self.b).cross(self.a - self.b).normalize();

        let mut vertices = [
            Vertex {
                position: self.a.into(),
                normal: normal.into(),
                tex_uv: [0.0, 0.0],
                ..Default::default()
            },
            Vertex {
                position: self.b.into(),
                normal: normal.into(),
                tex_uv: [1.0, 0.0],
                ..Default::default()
            },
            Vertex {
                position: self.c.into(),
                normal: normal.into(),
                tex_uv: [0.0, 1.0],
                ..Default::default()
            },
        ];

        let indices = [0, 1, 2];

        Ok(loader.create_mesh(&mut vertices, &indices))
    }
}
