use std::sync::OnceLock;

use bytemuck::{Pod, Zeroable};
use maple_engine::{
    Node, Scene,
    prelude::{EventReceiver, NodeTransform},
};
use maple_renderer::{
    core::{
        Buffer, DescriptorBindingType, DescriptorSet, DescriptorSetLayout,
        DescriptorSetLayoutDescriptor, LazyBuffer, LazyBufferable, RenderContext, StageFlags,
    },
    types::Vertex,
};
use parking_lot::RwLock;

use crate::components::material::MaterialProperties;

#[derive(Debug, Default, Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct Mesh3DUniformBufferData {
    pub model: [[f32; 4]; 4],
}

pub struct Mesh3D {
    transform: NodeTransform,
    children: Scene,
    events: EventReceiver,

    pub name: String,
    vertex_buffer: LazyBuffer<[Vertex]>,
    index_buffer: LazyBuffer<[u32]>,
    material: MaterialProperties,

    descriptor: RwLock<Option<DescriptorSet>>,
    uniform: LazyBuffer<Mesh3DUniformBufferData>,
    buffer_data: Mesh3DUniformBufferData,
}

impl Node for Mesh3D {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }

    fn get_events(&mut self) -> &mut EventReceiver {
        &mut self.events
    }

    fn get_children(&self) -> &Scene {
        &self.children
    }

    fn get_children_mut(&mut self) -> &mut Scene {
        &mut self.children
    }
}
//static so that we only allocate one
static LAYOUT: OnceLock<DescriptorSetLayout> = OnceLock::new();

impl Mesh3D {
    pub fn new(name: String, vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        let default_data = Mesh3DUniformBufferData::default();

        Self {
            transform: NodeTransform::default(),
            children: Scene::default(),
            events: EventReceiver::default(),

            name,
            vertex_buffer: RenderContext::create_vertex_buffer_lazy(&vertices),
            index_buffer: RenderContext::create_index_buffer_lazy(&indices),
            material: MaterialProperties::default(),

            buffer_data: Mesh3DUniformBufferData::default(),
            uniform: RenderContext::create_unifrom_buffer_lazy(&default_data),
            descriptor: RwLock::new(None),
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

    fn get_uniform(&self) -> Mesh3DUniformBufferData {
        let model = self.transform.world_space().matrix.to_cols_array_2d();

        Mesh3DUniformBufferData { model }
    }

    /// gets the mesh descriptor set (lazily allocated)
    pub fn get_descriptor(&self, rcx: &RenderContext) -> DescriptorSet {
        // update the uniform
        self.uniform.write(&self.get_uniform());

        // try to read
        {
            let read_guard = self.descriptor.read();
            if let Some(d) = &*read_guard {
                rcx.sync_lazy_buffer(&self.uniform);
                return d.clone();
            }
        }

        // not allocated yet
        let mut write_guard = self.descriptor.write();
        let layout = Self::layout(&rcx);
        let buffer = rcx.get_buffer(&self.uniform);
        let set = rcx.build_descriptor_set(DescriptorSet::builder(layout).uniform(0, &buffer));

        *write_guard = Some(set.clone());
        set.clone()
    }

    pub fn layout(rcx: &RenderContext) -> &DescriptorSetLayout {
        LAYOUT.get_or_init(|| {
            rcx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
                label: Some("Mesh"),
                visibility: StageFlags::VERTEX | StageFlags::FRAGMENT,
                layout: &[DescriptorBindingType::UniformBuffer],
            })
        })
    }
}
