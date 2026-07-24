use glam::{Quat, Vec2, Vec3};
use maple_engine::asset::{Asset, AssetLibrary, IntoAsset, LoadErr};

use crate::{assets::mesh::Mesh3D, math::Vertex};

/// Describes a plane shape for constructing a [`Mesh3D`]
pub struct Plane {
    pub normal: Vec3,
    pub size: Vec2,
    pub subdivisions: u32,
}

impl Plane {
    pub fn size(mut self, size: impl Into<Vec2>) -> Self {
        self.size = size.into();
        self
    }

    pub fn subdivisions(mut self, subdivisions: u32) -> Self {
        self.subdivisions = subdivisions;
        self
    }
}

impl Default for Plane {
    fn default() -> Self {
        Plane {
            normal: Vec3::Y,
            size: Vec2 { x: 1.0, y: 1.0 },
            subdivisions: 0,
        }
    }
}

impl IntoAsset<Mesh3D> for Plane {
    fn into_asset(
        self,
        loader: &<Mesh3D as Asset>::Loader,
        _library: &AssetLibrary,
    ) -> Result<Mesh3D, LoadErr> {
        let z_vertex_count = self.subdivisions + 2;
        let x_vertex_count = self.subdivisions + 2;
        let num_vertices = (z_vertex_count * x_vertex_count) as usize;
        let num_indices = ((z_vertex_count - 1) * (x_vertex_count - 1) * 6) as usize;

        let mut vertices: Vec<Vertex> = Vec::with_capacity(num_vertices);
        let mut indices: Vec<u32> = Vec::with_capacity(num_indices);

        let rotation = Quat::from_rotation_arc(Vec3::Y, self.normal);
        let size = self.size;

        for z in 0..z_vertex_count {
            for x in 0..x_vertex_count {
                let tx = x as f32 / (x_vertex_count - 1) as f32;
                let tz = z as f32 / (z_vertex_count - 1) as f32;
                let pos = rotation * Vec3::new((-0.5 + tx) * size.x, 0.0, (-0.5 + tz) * size.y);
                vertices.push(Vertex {
                    position: pos.to_array(),
                    normal: self.normal.to_array(),
                    tex_uv: [tx, tz],
                    // tangent and bitangent are calculated on creation of mesh
                    tangent: [0.0, 0.0, 0.0],
                    bitangent: [0.0, 0.0, 0.0],
                })
            }
        }

        for z in 0..z_vertex_count - 1 {
            for x in 0..x_vertex_count - 1 {
                let quad = z * x_vertex_count + x;
                indices.push(quad + x_vertex_count + 1);
                indices.push(quad + 1);
                indices.push(quad + x_vertex_count);
                indices.push(quad);
                indices.push(quad + x_vertex_count);
                indices.push(quad + 1);
            }
        }

        Ok(loader.create_mesh(&mut vertices, &indices))
    }
}
