use std::sync::{Arc, OnceLock};

use bytemuck::{Pod, Zeroable};
use maple_engine::{
    Buildable, Builder, Node,
    asset::AssetHandle,
    nodes::node_builder::NodePrototype,
    prelude::{NodeTransform, node_transform::WorldTransform},
};
use maple_renderer::{
    core::{
        Buffer, DescriptorBindingType, DescriptorSet, DescriptorSetLayout,
        DescriptorSetLayoutDescriptor, LazyBuffer, RenderContext, StageFlags,
    },
    types::Vertex,
};

use crate::{assets::mesh::Mesh3D, math::AABB, prelude::Material};

#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Mesh3DUniformBufferData {
    pub model: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 4]; 4],
}

#[derive(Clone)]
pub struct MeshInstance3D {
    pub transform: NodeTransform,
    pub mesh: Option<AssetHandle<Mesh3D>>,
    pub material: Option<AssetHandle<Material>>,
    descriptor: Arc<OnceLock<DescriptorSet>>,
    buffer: Arc<OnceLock<Buffer<Mesh3DUniformBufferData>>>,
}

impl MeshInstance3D {
    pub fn get_uniform(&self) -> Mesh3DUniformBufferData {
        let model = self.transform.world_space().matrix.to_cols_array_2d();
        let normal_matrix = self
            .transform
            .world_space()
            .matrix
            .inverse()
            .transpose()
            .to_cols_array_2d();

        Mesh3DUniformBufferData {
            model,
            normal_matrix,
        }
    }
    pub fn layout(rcx: &RenderContext) -> DescriptorSetLayout {
        rcx.get_or_create_layout(DescriptorSetLayoutDescriptor {
            label: Some("Mesh"),
            visibility: StageFlags::VERTEX,
            layout: &[
                DescriptorBindingType::UniformBuffer, // transforms
            ],
        })
    }

    pub fn get_descriptor(&self, rcx: &RenderContext) -> DescriptorSet {
        let buffer = self
            .buffer
            .get_or_init(|| rcx.device().create_uniform_buffer(&self.get_uniform()));

        rcx.queue().write_buffer(buffer, &self.get_uniform());

        self.descriptor
            .get_or_init(|| {
                rcx.device().build_descriptor_set(
                    DescriptorSet::builder(&Self::layout(rcx)).uniform(0, buffer),
                )
            })
            .clone()
    }
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
            descriptor: Arc::new(OnceLock::new()),
            buffer: Arc::new(OnceLock::new()),
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
