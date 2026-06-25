use bytemuck::{Pod, Zeroable};
use maple_engine::{
    Buildable, Builder, Node, asset::AssetHandle, nodes::node_builder::NodePrototype,
    prelude::NodeTransform,
};
use maple_renderer::{core::LazyBuffer, types::Vertex};

use crate::{assets::mesh::Mesh3D, math::AABB, prelude::Material};

#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Mesh3DUniformBufferData {
    pub model: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 4]; 4],
}

pub struct MeshInstance3D {
    pub transform: NodeTransform,
    pub mesh: Option<AssetHandle<Mesh3D>>,
    pub material: Option<AssetHandle<Material>>,
}

impl Node for MeshInstance3D {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }
}

#[derive(Default)]
pub struct MeshInstance3DBuilder {
    prototype: NodePrototype,
    mesh: Option<AssetHandle<Mesh3D>>,
    material: Option<AssetHandle<Material>>,
}

impl Buildable for MeshInstance3D {
    type Builder = MeshInstance3DBuilder;
    fn builder() -> Self::Builder {
        MeshInstance3DBuilder::default()
    }
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
            material: self.material,
        }
    }
}

impl MeshInstance3DBuilder {
    pub fn mesh(mut self, mesh: AssetHandle<Mesh3D>) -> Self {
        self.mesh = Some(mesh);
        self
    }

    pub fn material(mut self, material: AssetHandle<Material>) -> Self {
        self.material = Some(material);
        self
    }
}
