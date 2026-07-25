//! trait for vertex data

use bytemuck::{Pod, Zeroable};
use wgpu::{BufferAddress, VertexBufferLayout};

pub use wgpu::VertexAttribute;
pub use wgpu::VertexFormat;
pub use wgpu::vertex_attr_array;

/// layout of the Vertex for constructing a [`crate::core::pipeline::RenderPipeline`]
pub trait VertexLayout: Pod + Zeroable {
    /// attributes of the vertex
    const ATTRS: &'static [VertexAttribute];
    /// buffer layout of the vertex
    fn buffer_layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::ATTRS,
        }
    }
}
