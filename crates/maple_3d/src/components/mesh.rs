use maple_engine::utils::Debug;
use maple_renderer::{
    core::{Buffer, RenderContext},
    types::{LazyBuffer, Vertex, lazy_buffer::LazyArrayBuffer},
};

pub struct Mesh {
    pub name: String,
    vertex_buffer: LazyArrayBuffer<Vertex>,
    index_buffer: LazyArrayBuffer<Vertex>,
}

impl Mesh {
    fn new(name: String, vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self {
            name,
            vertex_buffer: LazyBuffer::new(vertices),
            index_buffer: LazyBuffer::new(indices),
        }
    }

    /// grabs the meshes vertices if they have been created if not it creates them with the
    /// renderer
    pub fn get_vertex_buffer(&self, rcx: &RenderContext) -> Option<Buffer<[Vertex]>> {
        self.vertex_buffer
            .get_buffer(|data| rcx.create_vertex_buffer(data))
    }

    /// grabs the meshes indices if they have been created if not it creates them with the
    /// renderer
    pub fn get_index_buffer(&self, rcx: &RenderContext) -> Option<Buffer<[u32]>> {
        self.index_buffer
            .get_buffer(|data| rcx.create_index_buffer(data))
    }
}
