use maple_engine::utils::Debug;
use maple_renderer::{
    core::{Buffer, LazyBuffer, RenderContext},
    types::Vertex,
};

use crate::components::material::MaterialProperties;

pub struct Mesh {
    pub name: String,
    vertex_buffer: LazyBuffer<[Vertex]>,
    index_buffer: LazyBuffer<[u32]>,
    material: MaterialProperties,
}

impl Mesh {
    pub fn new(name: String, vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self {
            name,
            vertex_buffer: RenderContext::create_vertex_buffer_lazy(&vertices),
            index_buffer: RenderContext::create_index_buffer_lazy(&indices),
            material: MaterialProperties::default(),
        }
    }

    /// grabs the meshes vertices if they have been created if not it creates them with the
    /// renderer
    pub fn get_vertex_buffer(&self, rcx: &RenderContext) -> Buffer<[Vertex]> {
        rcx.get_buffer(&self.vertex_buffer)
    }

    /// grabs the meshes indices if they have been created if not it creates them with the
    /// renderer
    pub fn get_index_buffer(&self, rcx: &RenderContext) -> Buffer<[u32]> {
        rcx.get_buffer(&self.index_buffer)
    }

    pub fn get_material(&self) -> &MaterialProperties {
        &self.material
    }
}
