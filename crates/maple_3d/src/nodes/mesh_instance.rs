use bytemuck::{Pod, Zeroable};
use maple_engine::{
    Builder, Node, asset::AssetHandle, nodes::node_builder::NodePrototype, prelude::NodeTransform,
};
use maple_renderer::{core::LazyBuffer, types::Vertex};

use crate::{assets::mesh::Mesh3D, math::AABB};

#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Mesh3DUniformBufferData {
    pub model: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 4]; 4],
}

/// Holds the LazyBuffers for a primitive mesh so they can be reused across instances
#[derive(Clone)]
pub struct PrimitiveMeshData {
    pub vertex_buffer: LazyBuffer<[Vertex]>,
    pub index_buffer: LazyBuffer<[u32]>,
    pub aabb: AABB,
}

pub struct MeshInstance3D {
    pub transform: NodeTransform,
    pub mesh: Option<AssetHandle<Mesh3D>>,
}

impl Node for MeshInstance3D {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }
}

pub struct MeshInstance3DBuilder {
    prototype: NodePrototype,
    mesh: Option<AssetHandle<Mesh3D>>,
}

impl Builder for MeshInstance3DBuilder {
    type Node = MeshInstance3D;
    fn prototype(&mut self) -> &mut NodePrototype {
        &mut self.prototype
    }

    fn build(self) -> Self::Node {
        Self::Node {
            transform: self.prototype.transform,
            mesh: self.mesh,
        }
    }
}

impl MeshInstance3DBuilder {
    pub fn mesh(mut self, mesh: AssetHandle<Mesh3D>) -> Self {
        self.mesh = Some(mesh);
        self
    }
}
