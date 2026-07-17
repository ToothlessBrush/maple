use bytemuck::{Pod, Zeroable};
use maple_renderer::types::vertex::{VertexAttribute, VertexLayout, vertex_attr_array};

#[derive(Default, Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],

    pub normal: [f32; 3],

    pub tex_uv: [f32; 2],

    pub tangent: [f32; 3],

    pub bitangent: [f32; 3],
}

impl VertexLayout for Vertex {
    const ATTRS: &'static [VertexAttribute] = &vertex_attr_array![
        0 => Float32x3, // position
        1 => Float32x3, // normal
        2 => Float32x2, // tex_uv
        3 => Float32x3, // tangent
        4 => Float32x3, // bitangent
    ];
}
