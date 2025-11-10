use std::sync::OnceLock;

use bytemuck::{Pod, Zeroable};
use maple_engine::{
    Buildable, Builder, Node, Scene,
    nodes::node_builder::NodePrototype,
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
    pub normal_matrix: [[f32; 4]; 4],
}

pub struct Mesh3D {
    pub transform: NodeTransform,
    pub children: Scene,
    events: EventReceiver,

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
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        let default_data = Mesh3DUniformBufferData::default();

        Self {
            transform: NodeTransform::default(),
            children: Scene::default(),
            events: EventReceiver::default(),

            vertex_buffer: RenderContext::create_vertex_buffer_lazy(&vertices),
            index_buffer: RenderContext::create_index_buffer_lazy(&indices),
            material: MaterialProperties::default(),

            buffer_data: Mesh3DUniformBufferData::default(),
            uniform: RenderContext::create_unifrom_buffer_lazy(&default_data),
            descriptor: RwLock::new(None),
        }
    }

    /// Creates a unit cube centered at the origin with side length 1.0
    pub fn cube() -> Mesh3DBuilder {
        // Define the 8 vertices of a cube
        let vertices = vec![
            // Front face (z = 0.5)
            Vertex {
                position: [-0.5, -0.5, 0.5],
                normal: [0.0, 0.0, 1.0],
                tex_uv: [0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.5],
                normal: [0.0, 0.0, 1.0],
                tex_uv: [1.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.5],
                normal: [0.0, 0.0, 1.0],
                tex_uv: [1.0, 1.0],
                tangent: [0.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, 0.5],
                normal: [0.0, 0.0, 1.0],
                tex_uv: [0.0, 1.0],
                tangent: [0.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
            },
            // Back face (z = -0.5)
            Vertex {
                position: [0.5, -0.5, -0.5],
                normal: [0.0, 0.0, -1.0],
                tex_uv: [0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, -0.5, -0.5],
                normal: [0.0, 0.0, -1.0],
                tex_uv: [1.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, -0.5],
                normal: [0.0, 0.0, -1.0],
                tex_uv: [1.0, 1.0],
                tangent: [0.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, -0.5],
                normal: [0.0, 0.0, -1.0],
                tex_uv: [0.0, 1.0],
                tangent: [0.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
            },
            // Right face (x = 0.5)
            Vertex {
                position: [0.5, -0.5, 0.5],
                normal: [1.0, 0.0, 0.0],
                tex_uv: [0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, -0.5],
                normal: [1.0, 0.0, 0.0],
                tex_uv: [1.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, -0.5],
                normal: [1.0, 0.0, 0.0],
                tex_uv: [1.0, 1.0],
                tangent: [0.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.5],
                normal: [1.0, 0.0, 0.0],
                tex_uv: [0.0, 1.0],
                tangent: [0.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
            },
            // Left face (x = -0.5)
            Vertex {
                position: [-0.5, -0.5, -0.5],
                normal: [-1.0, 0.0, 0.0],
                tex_uv: [0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, -0.5, 0.5],
                normal: [-1.0, 0.0, 0.0],
                tex_uv: [1.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, 0.5],
                normal: [-1.0, 0.0, 0.0],
                tex_uv: [1.0, 1.0],
                tangent: [0.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, -0.5],
                normal: [-1.0, 0.0, 0.0],
                tex_uv: [0.0, 1.0],
                tangent: [0.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
            },
            // Top face (y = 0.5)
            Vertex {
                position: [-0.5, 0.5, 0.5],
                normal: [0.0, 1.0, 0.0],
                tex_uv: [0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.5],
                normal: [0.0, 1.0, 0.0],
                tex_uv: [1.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, -0.5],
                normal: [0.0, 1.0, 0.0],
                tex_uv: [1.0, 1.0],
                tangent: [0.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, -0.5],
                normal: [0.0, 1.0, 0.0],
                tex_uv: [0.0, 1.0],
                tangent: [0.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
            },
            // Bottom face (y = -0.5)
            Vertex {
                position: [-0.5, -0.5, -0.5],
                normal: [0.0, -1.0, 0.0],
                tex_uv: [0.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, -0.5],
                normal: [0.0, -1.0, 0.0],
                tex_uv: [1.0, 0.0],
                tangent: [0.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.5],
                normal: [0.0, -1.0, 0.0],
                tex_uv: [1.0, 1.0],
                tangent: [0.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
            },
            Vertex {
                position: [-0.5, -0.5, 0.5],
                normal: [0.0, -1.0, 0.0],
                tex_uv: [0.0, 1.0],
                tangent: [0.0, 0.0, 0.0],
                bitangent: [0.0, 0.0, 0.0],
            },
        ];

        // Define indices for 6 faces (2 triangles per face)
        let indices = vec![
            // Front face
            0, 1, 2, 2, 3, 0, // Back face
            4, 5, 6, 6, 7, 4, // Right face
            8, 9, 10, 10, 11, 8, // Left face
            12, 13, 14, 14, 15, 12, // Top face
            16, 17, 18, 18, 19, 16, // Bottom face
            20, 21, 22, 22, 23, 20,
        ];

        Mesh3DBuilder::new(vertices, indices)
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
        let layout = Self::layout(rcx);
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

impl Buildable for Mesh3D {
    type Builder = Mesh3DBuilder;

    fn builder() -> Self::Builder {
        Mesh3DBuilder {
            proto: NodePrototype::default(),
            vertices: vec![],
            indices: vec![],
            material: MaterialProperties::default(),
        }
    }
}

pub struct Mesh3DBuilder {
    proto: NodePrototype,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    material: MaterialProperties,
}

impl Builder for Mesh3DBuilder {
    type Node = Mesh3D;

    fn prototype(&mut self) -> &mut NodePrototype {
        &mut self.proto
    }

    fn build(self) -> Self::Node {
        let default_data = Mesh3DUniformBufferData::default();

        Mesh3D {
            transform: self.proto.transform,
            children: self.proto.children,
            events: self.proto.events,
            vertex_buffer: RenderContext::create_vertex_buffer_lazy(&self.vertices),
            index_buffer: RenderContext::create_index_buffer_lazy(&self.indices),
            material: self.material,

            buffer_data: Mesh3DUniformBufferData::default(),
            uniform: RenderContext::create_unifrom_buffer_lazy(&default_data),
            descriptor: RwLock::new(None),
        }
    }
}

impl Mesh3DBuilder {
    pub fn material(mut self, material: MaterialProperties) -> Self {
        self.material = material;
        self
    }

    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self {
            proto: NodePrototype::default(),
            vertices,
            indices,
            material: MaterialProperties::default(),
        }
    }
}
