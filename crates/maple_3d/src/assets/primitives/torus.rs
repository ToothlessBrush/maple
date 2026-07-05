use std::{f32::consts::PI, sync::Arc};

use maple_engine::asset::IntoAsset;
use maple_renderer::types::Vertex;

use crate::assets::mesh::Mesh3D;

pub struct Torus {
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub sides: u32,
    pub rings: u32,
}

impl IntoAsset<Mesh3D> for Torus {
    fn into_asset(
        self,
        loader: &<Mesh3D as maple_engine::asset::Asset>::Loader,
        _library: &maple_engine::prelude::AssetLibrary,
    ) -> Result<std::sync::Arc<Mesh3D>, maple_engine::asset::LoadErr> {
        let ring_radius = (self.outer_radius - self.inner_radius) / 2.0;
        let num_vertices_per_row = self.sides + 1;
        let num_vertices_per_column = self.rings + 1;

        let mut vertices: Vec<Vertex> = Vec::new();

        let vertical_angular_stride = (PI * 2.0) / self.rings as f32;
        let horizontal_angular_stride = (PI * 2.0) / self.sides as f32;

        for vertical_index in 0..num_vertices_per_column {
            let theta = vertical_angular_stride * vertical_index as f32;

            for horizontal_index in 0..num_vertices_per_row {
                let phi = horizontal_angular_stride * horizontal_index as f32;

                let x = f32::cos(theta) * (self.outer_radius + ring_radius * f32::cos(phi));
                let z = f32::sin(theta) * (self.outer_radius + ring_radius * f32::cos(phi));
                let y = ring_radius * f32::sin(phi);
                let normal = [
                    f32::cos(theta) * f32::cos(phi),
                    f32::sin(phi),
                    f32::sin(theta) * f32::cos(phi),
                ];

                let tangent = [-f32::sin(theta), 0.0, f32::cos(theta)];

                let bitangent = [
                    -f32::cos(theta) * f32::sin(phi),
                    f32::cos(phi),
                    -f32::sin(theta) * f32::sin(phi),
                ];
                vertices.push(Vertex {
                    position: [x, y, z],
                    normal: normal,
                    tex_uv: [
                        horizontal_index as f32 / self.sides as f32,
                        vertical_index as f32 / self.rings as f32,
                    ],
                    tangent: tangent,
                    bitangent: bitangent,
                });
            }
        }

        let mut indices: Vec<u32> = Vec::new();

        for vertical_index in 0..self.rings {
            for horizontal_index in 0..self.sides {
                let lt = horizontal_index + vertical_index * num_vertices_per_row;
                let rt = (horizontal_index + 1) + vertical_index * num_vertices_per_row;

                let lb = horizontal_index + (vertical_index + 1) * num_vertices_per_row;
                let rb = (horizontal_index + 1) + (vertical_index + 1) * num_vertices_per_row;

                indices.append(&mut vec![lt, rt, lb, rt, rb, lb]);
            }
        }

        let mesh = loader.create_mesh(vertices, indices);

        Ok(Arc::new(mesh))
    }
}
