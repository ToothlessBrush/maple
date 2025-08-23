use std::sync::Arc;

use crate::{core::buffer::Buffer, types::Vertex};

pub trait Drawable {
    fn vertex_buffer(&self) -> Arc<Buffer<Vertex>>;
    fn index_buffer(&self) -> Arc<Buffer<u32>>;
}
