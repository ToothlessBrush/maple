use maple_engine::{
    asset::{Asset, AssetLoader},
    prelude::node_transform::WorldTransform,
};
use maple_renderer::core::{Buffer, RenderDevice};
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator,
};

use crate::math::{AABB, Vertex};

pub struct Mesh3DLoader {
    device: RenderDevice,
}

impl AssetLoader for Mesh3DLoader {
    type Asset = Mesh3D;
}

impl Mesh3DLoader {
    pub(crate) fn new(device: RenderDevice) -> Self {
        Self { device }
    }

    pub fn calculate_tangents(vertices: &mut [Vertex], indices: &[u32]) {
        // Check if we have valid UVs (not all zeros)
        let has_valid_uvs = vertices
            .iter()
            .any(|v| v.tex_uv[0].abs() > 1e-6 || v.tex_uv[1].abs() > 1e-6);

        if !has_valid_uvs {
            // Generate tangent space from normals only
            vertices.par_iter_mut().for_each(|vertex| {
                let n = vertex.normal;

                // Create an arbitrary perpendicular vector for the tangent
                // Choose a vector that's not parallel to the normal
                let tangent = if n[0].abs() > 0.9 {
                    // Normal is mostly along X, use Y axis
                    [0.0, 1.0, 0.0]
                } else {
                    // Use X axis
                    [1.0, 0.0, 0.0]
                };

                // Gram-Schmidt orthogonalize tangent against normal
                let dot_nt = n[0] * tangent[0] + n[1] * tangent[1] + n[2] * tangent[2];
                let ortho_t = [
                    tangent[0] - n[0] * dot_nt,
                    tangent[1] - n[1] * dot_nt,
                    tangent[2] - n[2] * dot_nt,
                ];

                // Normalize tangent
                let len_t =
                    (ortho_t[0] * ortho_t[0] + ortho_t[1] * ortho_t[1] + ortho_t[2] * ortho_t[2])
                        .sqrt();
                vertex.tangent = [ortho_t[0] / len_t, ortho_t[1] / len_t, ortho_t[2] / len_t];

                // Bitangent = cross(normal, tangent)
                vertex.bitangent = [
                    n[1] * vertex.tangent[2] - n[2] * vertex.tangent[1],
                    n[2] * vertex.tangent[0] - n[0] * vertex.tangent[2],
                    n[0] * vertex.tangent[1] - n[1] * vertex.tangent[0],
                ];
            });
            return;
        }

        // Initialize all tangents and bitangents to zero
        vertices.par_iter_mut().for_each(|vertex| {
            vertex.tangent = [0.0, 0.0, 0.0];
            vertex.bitangent = [0.0, 0.0, 0.0];
        });

        // Pre-calculate tangent/bitangent contributions per triangle
        let triangle_contributions: Vec<_> = (0..indices.len())
            .into_par_iter()
            .step_by(3)
            .map(|i| {
                let i0 = indices[i] as usize;
                let i1 = indices[i + 1] as usize;
                let i2 = indices[i + 2] as usize;

                let v0 = &vertices[i0];
                let v1 = &vertices[i1];
                let v2 = &vertices[i2];

                // Position deltas
                let edge1 = [
                    v1.position[0] - v0.position[0],
                    v1.position[1] - v0.position[1],
                    v1.position[2] - v0.position[2],
                ];
                let edge2 = [
                    v2.position[0] - v0.position[0],
                    v2.position[1] - v0.position[1],
                    v2.position[2] - v0.position[2],
                ];

                // UV deltas
                let delta_uv1 = [v1.tex_uv[0] - v0.tex_uv[0], v1.tex_uv[1] - v0.tex_uv[1]];
                let delta_uv2 = [v2.tex_uv[0] - v0.tex_uv[0], v2.tex_uv[1] - v0.tex_uv[1]];

                // Calculate tangent and bitangent
                let det = delta_uv1[0] * delta_uv2[1] - delta_uv1[1] * delta_uv2[0];
                let r = if det.abs() > 1e-6 { 1.0 / det } else { 0.0 };

                let tangent = [
                    r * (delta_uv2[1] * edge1[0] - delta_uv1[1] * edge2[0]),
                    r * (delta_uv2[1] * edge1[1] - delta_uv1[1] * edge2[1]),
                    r * (delta_uv2[1] * edge1[2] - delta_uv1[1] * edge2[2]),
                ];

                let bitangent = [
                    r * (-delta_uv2[0] * edge1[0] + delta_uv1[0] * edge2[0]),
                    r * (-delta_uv2[0] * edge1[1] + delta_uv1[0] * edge2[1]),
                    r * (-delta_uv2[0] * edge1[2] + delta_uv1[0] * edge2[2]),
                ];

                (i0, i1, i2, tangent, bitangent)
            })
            .collect();

        // Accumulate contributions (must be sequential due to race conditions)
        for (i0, i1, i2, tangent, bitangent) in triangle_contributions {
            vertices[i0].tangent[0] += tangent[0];
            vertices[i0].tangent[1] += tangent[1];
            vertices[i0].tangent[2] += tangent[2];

            vertices[i1].tangent[0] += tangent[0];
            vertices[i1].tangent[1] += tangent[1];
            vertices[i1].tangent[2] += tangent[2];

            vertices[i2].tangent[0] += tangent[0];
            vertices[i2].tangent[1] += tangent[1];
            vertices[i2].tangent[2] += tangent[2];

            vertices[i0].bitangent[0] += bitangent[0];
            vertices[i0].bitangent[1] += bitangent[1];
            vertices[i0].bitangent[2] += bitangent[2];

            vertices[i1].bitangent[0] += bitangent[0];
            vertices[i1].bitangent[1] += bitangent[1];
            vertices[i1].bitangent[2] += bitangent[2];

            vertices[i2].bitangent[0] += bitangent[0];
            vertices[i2].bitangent[1] += bitangent[1];
            vertices[i2].bitangent[2] += bitangent[2];
        }

        // Normalize and orthogonalize in parallel
        vertices.par_iter_mut().for_each(|vertex| {
            let n = vertex.normal;
            let t = vertex.tangent;

            // Gram-Schmidt orthogonalize
            let dot_nt = n[0] * t[0] + n[1] * t[1] + n[2] * t[2];

            let ortho_t = [
                t[0] - n[0] * dot_nt,
                t[1] - n[1] * dot_nt,
                t[2] - n[2] * dot_nt,
            ];

            // Normalize tangent
            let len_t =
                (ortho_t[0] * ortho_t[0] + ortho_t[1] * ortho_t[1] + ortho_t[2] * ortho_t[2])
                    .sqrt();
            if len_t > 1e-6 {
                vertex.tangent = [ortho_t[0] / len_t, ortho_t[1] / len_t, ortho_t[2] / len_t];
            } else {
                // Fallback for degenerate cases
                if n[0].abs() > 0.9 {
                    vertex.tangent = [0.0, 1.0, 0.0];
                } else {
                    vertex.tangent = [1.0, 0.0, 0.0];
                }
            }

            // Normalize bitangent
            let b = vertex.bitangent;
            let len_b = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
            if len_b > 1e-6 {
                vertex.bitangent = [b[0] / len_b, b[1] / len_b, b[2] / len_b];
            } else {
                // Calculate bitangent from cross product
                vertex.bitangent = [
                    n[1] * vertex.tangent[2] - n[2] * vertex.tangent[1],
                    n[2] * vertex.tangent[0] - n[0] * vertex.tangent[2],
                    n[0] * vertex.tangent[1] - n[1] * vertex.tangent[0],
                ];
            }
        });
    }

    pub fn create_mesh(&self, mut vertices: &mut [Vertex], indices: &[u32]) -> Mesh3D {
        Self::calculate_tangents(&mut vertices, &indices);
        Mesh3D::new(&self.device, vertices, indices)
    }
}

/// Mesh3D is a [`Asset`] that reprensents an objects shape on the gpu
///
/// it contains a refrence to vertices and indices
#[derive(Debug, Clone)]
pub struct Mesh3D {
    // pub transform: NodeTransform,
    vertex_buffer: Buffer<[Vertex]>,
    index_buffer: Buffer<[u32]>,

    aabb: AABB,
}

impl Asset for Mesh3D {
    type Loader = Mesh3DLoader;
}

impl Mesh3D {
    pub fn new(device: &RenderDevice, vertices: &[Vertex], indices: &[u32]) -> Self {
        let aabb = AABB::from_vertices(&vertices);

        Self {
            // transform: NodeTransform::default(),
            vertex_buffer: device.create_vertex_buffer(&vertices),
            index_buffer: device.create_index_buffer(&indices),
            // material: MaterialProperties::default(),
            aabb,
        }
    }

    /// Creates a mesh from existing buffers (useful for sharing buffers between instances)
    pub fn from_buffers(
        vertex_buffer: Buffer<[Vertex]>,
        index_buffer: Buffer<[u32]>,
        aabb: AABB,
    ) -> Self {
        Self {
            // transform: NodeTransform::default(),
            vertex_buffer,
            index_buffer,

            aabb,
        }
    }

    /// grabs the meshes vertices if they have been created if not it creates them with the
    /// renderer
    pub fn get_vertex_buffer(&self) -> &Buffer<[Vertex]> {
        &self.vertex_buffer
    }

    /// grabs the meshes indices if they have been created if not it creates them with the
    /// renderer
    pub fn get_index_buffer(&self) -> &Buffer<[u32]> {
        &self.index_buffer
    }

    // get the bounding box in world space
    pub fn world_aabb(&self, transform: WorldTransform) -> AABB {
        self.aabb.transform(&transform.matrix)
    }
}
