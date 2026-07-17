use bytemuck::{Pod, Zeroable};
use wgpu::{BufferAddress, VertexBufferLayout};

pub use wgpu::VertexAttribute;
pub use wgpu::VertexFormat;
pub use wgpu::vertex_attr_array;

pub trait VertexLayout: Pod + Zeroable {
    const ATTRS: &'static [VertexAttribute];
    fn buffer_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::ATTRS,
        }
    }
}
