use std::sync::Arc;

use crate::{
    core::buffer::{Buffer, BufferBackend},
    types::Vertex,
};
use anyhow::anyhow;
use vulkano::{
    buffer::{BufferContents, IndexBuffer, Subbuffer},
    pipeline::graphics::vertex_input::VertexBuffersCollection,
};

pub struct VulkanBuffer<T: ?Sized + BufferContents> {
    pub buffer: Subbuffer<T>,
}

impl<T: ?Sized + BufferContents> Clone for VulkanBuffer<T> {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
        }
    }
}

impl<T: ?Sized + BufferContents> From<Buffer<T>> for VulkanBuffer<T> {
    fn from(value: Buffer<T>) -> Self {
        if let BufferBackend::VK(b) = value.inner {
            b
        } else {
            unreachable!("mismatched backend apis ")
        }
    }
}

impl VertexBuffersCollection for VulkanBuffer<[Vertex]> {
    fn into_vec(self) -> Vec<Subbuffer<[u8]>> {
        vec![self.buffer.as_bytes().clone()]
    }
}

impl From<VulkanBuffer<[u32]>> for IndexBuffer {
    fn from(value: VulkanBuffer<[u32]>) -> Self {
        IndexBuffer::U32(value.buffer.clone())
    }
}
