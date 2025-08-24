use wgpu::RenderPass;

use crate::{
    core::{buffer::Buffer, descriptor_set::DescriptorSet},
    types::Vertex,
};

/// builder for a frame use this to bind buffers, descriptor sets, or anything else frame related
///
/// since the frame contains a refrence to the command encoder we need its lifetime
pub struct FrameBuilder<'encoder> {
    pub(crate) backend: RenderPass<'encoder>,
    index_count: u32,
    vertex_count: u32,
}

impl<'encoder> FrameBuilder<'encoder> {
    pub(crate) fn new(backend: RenderPass<'encoder>) -> Self {
        FrameBuilder {
            backend,
            index_count: 0,
            vertex_count: 0,
        }
    }

    /// vertex buffer for the next draw call
    pub fn bind_vertex_buffer(&mut self, vertex_buffer: &Buffer<[Vertex]>) -> &mut Self {
        self.backend
            .set_vertex_buffer(0, vertex_buffer.buffer.slice(..));

        self.vertex_count = vertex_buffer.len() as u32;

        self
    }

    /// index buffer for the next draw_indexed call
    pub fn bind_index_buffer(&mut self, index_buffer: &Buffer<[u32]>) -> &mut Self {
        self.backend
            .set_index_buffer(index_buffer.buffer.slice(..), wgpu::IndexFormat::Uint32);

        self.index_count = index_buffer.len() as u32;

        self
    }

    // set a descriptor set must be in the pipeline layout
    pub fn bind_descriptor_set(&mut self, set: u32, descriptor_set: &DescriptorSet) -> &mut Self {
        self.backend
            .set_bind_group(set, &descriptor_set.backend, &[]);

        self
    }

    pub fn debug_marker(&mut self, label: &str) -> &mut Self {
        self.backend.insert_debug_marker(label);

        self
    }

    /// draw the last bound indicies
    pub fn draw_indexed(&mut self) -> &mut Self {
        self.backend.draw_indexed(0..self.index_count, 0, 0..1);

        self
    }

    /// draw the last bound verticies
    pub fn draw(&mut self) -> &mut Self {
        self.backend.draw(0..self.vertex_count, 0..1);

        self
    }
}
