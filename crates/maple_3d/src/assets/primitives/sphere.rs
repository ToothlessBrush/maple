use std::f32::consts::PI;

use maple_engine::asset::{IntoAsset, LoadErr};
use maple_renderer::types::Vertex;

use crate::assets::mesh::Mesh3D;

/// describes a sphere for constructing a [`crate::assets::mesh::Mesh3D`] asset.
#[derive(Debug)]
pub struct Sphere {
    /// distance from center to edge
    ///
    /// must be > 0
    pub radius: f32,
    /// how many sections wrap around the sphere
    ///
    /// must be >= 3
    pub sectors: u32,
    /// how many sections from bottom to top
    ///
    /// must be >= 2
    pub stacks: u32,
}

impl Sphere {
    /// distance from center to edge
    ///
    /// must be > 0
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// how many sections wrap around the sphere
    ///
    /// must be >= 3
    pub fn stacks(mut self, stacks: u32) -> Self {
        self.stacks = stacks;
        self
    }

    /// how many sections from bottom to top
    ///
    /// must be >= 2
    pub fn sectors(mut self, sectors: u32) -> Self {
        self.sectors = sectors;
        self
    }
}

impl Default for Sphere {
    fn default() -> Self {
        // copy blender defaults
        Sphere {
            radius: 1.0,
            sectors: 32,
            stacks: 16,
        }
    }
}

impl IntoAsset<Mesh3D> for Sphere {
    fn into_asset(
        self,
        loader: &<Mesh3D as maple_engine::asset::Asset>::Loader,
        _library: &maple_engine::prelude::AssetLibrary,
    ) -> Result<Mesh3D, maple_engine::asset::LoadErr> {
        // alg from here: https://www.songho.ca/opengl/gl_sphere.html

        if self.radius <= 0.0 {
            return Err(LoadErr::IntoAsset(format!(
                "sphere radius must be > 0, got {}",
                self.radius
            )));
        }
        if self.stacks < 2 {
            return Err(LoadErr::IntoAsset(
                "sphere stack count cannot be less then 2".into(),
            ));
        }
        if self.sectors < 3 {
            return Err(LoadErr::IntoAsset(
                "shpere sector count cannot be less then 3".into(),
            ));
        }

        let mut vertices: Vec<Vertex> = Vec::new();

        let length_inv = 1.0 / self.radius;

        let sector_step = 2.0 * PI / self.sectors as f32;
        let stack_step = PI / self.stacks as f32;

        for i in 0..=self.stacks {
            let stack_angle = PI / 2.0 - i as f32 * stack_step;
            let xy = self.radius * f32::cos(stack_angle);
            let y = self.radius * f32::sin(stack_angle);

            for j in 0..=self.sectors {
                let sector_angle = j as f32 * sector_step;

                // vertex pos
                let x = xy * f32::cos(sector_angle);
                let z = xy * f32::sin(sector_angle);

                // normal
                let nx = x * length_inv;
                let nz = z * length_inv;
                let ny = y * length_inv;

                // tex uv
                let s = j as f32 / self.sectors as f32;
                let t = i as f32 / self.stacks as f32;

                vertices.push(Vertex {
                    position: [x, y, z],
                    normal: [nx, ny, nz],
                    tex_uv: [s, t],
                    ..Default::default()
                });
            }
        }

        // generate CW index list of sphere triangles
        // k1--k1+1
        // |  / |
        // | /  |
        // k2--k2+1
        let mut indices: Vec<u32> = Vec::new();
        for i in 0..self.stacks {
            let k1 = i * (self.sectors + 1);
            let k2 = k1 + self.sectors + 1;

            for j in 0..self.sectors {
                let k1 = k1 + j;
                let k2 = k2 + j;

                if i != 0 {
                    indices.push(k1);
                    indices.push(k1 + 1);
                    indices.push(k2);
                }

                if i != (self.stacks - 1) {
                    indices.push(k1 + 1);
                    indices.push(k2 + 1);
                    indices.push(k2);
                }
            }
        }

        Ok(loader.create_mesh(vertices, indices))
    }
}
