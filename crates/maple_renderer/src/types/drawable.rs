use std::sync::Arc;

use vulkano::buffer::BufferContents;

use crate::{core::buffer::Buffer, types::Vertex};

pub trait Drawable {
    fn uniform<T: BufferContents + 'static>(&self) -> Arc<Buffer<T>>;
    fn vertex_buffer(&self) -> Arc<Buffer<Vertex>>;
    fn index_buffer(&self) -> Arc<Buffer<u32>>;
}
