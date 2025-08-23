use bytemuck::{AnyBitPattern, NoUninit, Pod, Zeroable};
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout};

#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],

    pub normal: [f32; 3],

    pub tex_uv: [f32; 2],
}

impl Vertex {
    pub const ATTRS: [VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x3, // pos
        1 => Float32x3, // normal
        2 => Float32x2, // uv
    ];

    pub const fn buffer_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRS,
        }
    }
}
