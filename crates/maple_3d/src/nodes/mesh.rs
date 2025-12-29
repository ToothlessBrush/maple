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
use rayon::iter::{
    IndexedParallelIterator, IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator,
};

use crate::components::material::MaterialProperties;

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
}

// Static storage for primitive meshes
static PRIMITIVE_CUBE: OnceLock<PrimitiveMeshData> = OnceLock::new();
static PRIMITIVE_SPHERE: OnceLock<PrimitiveMeshData> = OnceLock::new();
static PRIMITIVE_SMOOTH_SPHERE: OnceLock<PrimitiveMeshData> = OnceLock::new();
static PRIMITIVE_CYLINDER: OnceLock<PrimitiveMeshData> = OnceLock::new();
static PRIMITIVE_CONE: OnceLock<PrimitiveMeshData> = OnceLock::new();
static PRIMITIVE_PLANE: OnceLock<PrimitiveMeshData> = OnceLock::new();
static PRIMITIVE_PYRAMID: OnceLock<PrimitiveMeshData> = OnceLock::new();
static PRIMITIVE_TORUS: OnceLock<PrimitiveMeshData> = OnceLock::new();

pub struct Mesh3D {
    pub transform: NodeTransform,
    pub children: Scene,
    events: EventReceiver,

    vertex_buffer: LazyBuffer<[Vertex]>,
    index_buffer: LazyBuffer<[u32]>,
    material: MaterialProperties,

    descriptor: RwLock<Option<DescriptorSet>>,
    uniform: LazyBuffer<Mesh3DUniformBufferData>,
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

            uniform: RenderContext::create_unifrom_buffer_lazy(&default_data),
            descriptor: RwLock::new(None),
        }
    }

    /// Creates a mesh from existing buffers (useful for sharing buffers between instances)
    pub fn from_buffers(
        vertex_buffer: LazyBuffer<[Vertex]>,
        index_buffer: LazyBuffer<[u32]>,
        material: MaterialProperties,
    ) -> Self {
        let default_data = Mesh3DUniformBufferData::default();

        Self {
            transform: NodeTransform::default(),
            children: Scene::default(),
            events: EventReceiver::default(),

            vertex_buffer,
            index_buffer,
            material,

            uniform: RenderContext::create_unifrom_buffer_lazy(&default_data),
            descriptor: RwLock::new(None),
        }
    }

    /// creates an instance of the mesh
    ///
    /// Stores the same handles to vertex index and material data but has unique transform and
    /// uniform data
    pub fn instance(&self) -> Self {
        Self {
            transform: self.transform,
            children: Scene::default(),
            events: self.events.clone(),
            vertex_buffer: self.vertex_buffer.clone(), // should be refrence copy
            index_buffer: self.index_buffer.clone(),   // should be refrence copy
            material: self.material.clone(),           // should be refrence copy
            descriptor: RwLock::new(None),             // unique
            uniform: RenderContext::create_unifrom_buffer_lazy(&Mesh3DUniformBufferData::default()), // unique
        }
    }

    /// Creates a unit cube centered at the origin with side length 1.0
    /// Uses shared GPU buffers - cloning is cheap since LazyBuffer uses Arc internally
    pub fn cube() -> Mesh3DBuilder {
        const CUBE_BYTES: &[u8] = include_bytes!("../../res/primitives/cube.glb");
        let primitive = Self::get_primitive(&PRIMITIVE_CUBE, CUBE_BYTES);

        Mesh3DBuilder {
            proto: NodePrototype::default(),
            vertices: vec![],
            indices: vec![],
            material: MaterialProperties::default(),
            vertex_buffer: Some(primitive.vertex_buffer.clone()),
            index_buffer: Some(primitive.index_buffer.clone()),
        }
    }

    /// Creates a sphere
    /// Uses shared GPU buffers - cloning is cheap since LazyBuffer uses Arc internally
    pub fn sphere() -> Mesh3DBuilder {
        const SPHERE_BYTES: &[u8] = include_bytes!("../../res/primitives/sphere.glb");
        let primitive = Self::get_primitive(&PRIMITIVE_SPHERE, SPHERE_BYTES);

        Mesh3DBuilder {
            proto: NodePrototype::default(),
            vertices: vec![],
            indices: vec![],
            material: MaterialProperties::default(),
            vertex_buffer: Some(primitive.vertex_buffer.clone()),
            index_buffer: Some(primitive.index_buffer.clone()),
        }
    }

    /// Creates a smooth sphere
    /// Uses shared GPU buffers - cloning is cheap since LazyBuffer uses Arc internally
    pub fn smooth_sphere() -> Mesh3DBuilder {
        const SMOOTH_SPHERE_BYTES: &[u8] = include_bytes!("../../res/primitives/smooth_sphere.glb");
        let primitive = Self::get_primitive(&PRIMITIVE_SMOOTH_SPHERE, SMOOTH_SPHERE_BYTES);

        Mesh3DBuilder {
            proto: NodePrototype::default(),
            vertices: vec![],
            indices: vec![],
            material: MaterialProperties::default(),
            vertex_buffer: Some(primitive.vertex_buffer.clone()),
            index_buffer: Some(primitive.index_buffer.clone()),
        }
    }

    /// Creates a cylinder
    /// Uses shared GPU buffers - cloning is cheap since LazyBuffer uses Arc internally
    pub fn cylinder() -> Mesh3DBuilder {
        const CYLINDER_BYTES: &[u8] = include_bytes!("../../res/primitives/cylinder.glb");
        let primitive = Self::get_primitive(&PRIMITIVE_CYLINDER, CYLINDER_BYTES);

        Mesh3DBuilder {
            proto: NodePrototype::default(),
            vertices: vec![],
            indices: vec![],
            material: MaterialProperties::default(),
            vertex_buffer: Some(primitive.vertex_buffer.clone()),
            index_buffer: Some(primitive.index_buffer.clone()),
        }
    }

    /// Creates a cone
    /// Uses shared GPU buffers - cloning is cheap since LazyBuffer uses Arc internally
    pub fn cone() -> Mesh3DBuilder {
        const CONE_BYTES: &[u8] = include_bytes!("../../res/primitives/cone.glb");
        let primitive = Self::get_primitive(&PRIMITIVE_CONE, CONE_BYTES);

        Mesh3DBuilder {
            proto: NodePrototype::default(),
            vertices: vec![],
            indices: vec![],
            material: MaterialProperties::default(),
            vertex_buffer: Some(primitive.vertex_buffer.clone()),
            index_buffer: Some(primitive.index_buffer.clone()),
        }
    }

    /// Creates a plane
    /// Uses shared GPU buffers - cloning is cheap since LazyBuffer uses Arc internally
    pub fn plane() -> Mesh3DBuilder {
        const PLANE_BYTES: &[u8] = include_bytes!("../../res/primitives/plane.glb");
        let primitive = Self::get_primitive(&PRIMITIVE_PLANE, PLANE_BYTES);

        Mesh3DBuilder {
            proto: NodePrototype::default(),
            vertices: vec![],
            indices: vec![],
            material: MaterialProperties::default(),
            vertex_buffer: Some(primitive.vertex_buffer.clone()),
            index_buffer: Some(primitive.index_buffer.clone()),
        }
    }

    /// Creates a pyramid
    /// Uses shared GPU buffers - cloning is cheap since LazyBuffer uses Arc internally
    pub fn pyramid() -> Mesh3DBuilder {
        const PYRAMID_BYTES: &[u8] = include_bytes!("../../res/primitives/pyramid.glb");
        let primitive = Self::get_primitive(&PRIMITIVE_PYRAMID, PYRAMID_BYTES);

        Mesh3DBuilder {
            proto: NodePrototype::default(),
            vertices: vec![],
            indices: vec![],
            material: MaterialProperties::default(),
            vertex_buffer: Some(primitive.vertex_buffer.clone()),
            index_buffer: Some(primitive.index_buffer.clone()),
        }
    }

    /// Creates a torus
    /// Uses shared GPU buffers - cloning is cheap since LazyBuffer uses Arc internally
    pub fn torus() -> Mesh3DBuilder {
        const TORUS_BYTES: &[u8] = include_bytes!("../../res/primitives/torus.glb");
        let primitive = Self::get_primitive(&PRIMITIVE_TORUS, TORUS_BYTES);

        Mesh3DBuilder {
            proto: NodePrototype::default(),
            vertices: vec![],
            indices: vec![],
            material: MaterialProperties::default(),
            vertex_buffer: Some(primitive.vertex_buffer.clone()),
            index_buffer: Some(primitive.index_buffer.clone()),
        }
    }

    pub fn calculate_tangents(vertices: &mut [Vertex], indices: &[u32]) {
        // Check if we have valid UVs (not all zeros)
        let has_valid_uvs = vertices
            .iter()
            .any(|v| v.tex_uv[0].abs() > 1e-6 || v.tex_uv[1].abs() > 1e-6);

        if !has_valid_uvs {
            // Generate tangent space from normals only
            vertices.par_iter_mut().for_each(|vertex| {
                let n = vertex.normal;

                // Create an arbitrary perpendicular vector for the tangent
                // Choose a vector that's not parallel to the normal
                let tangent = if n[0].abs() > 0.9 {
                    // Normal is mostly along X, use Y axis
                    [0.0, 1.0, 0.0]
                } else {
                    // Use X axis
                    [1.0, 0.0, 0.0]
                };

                // Gram-Schmidt orthogonalize tangent against normal
                let dot_nt = n[0] * tangent[0] + n[1] * tangent[1] + n[2] * tangent[2];
                let ortho_t = [
                    tangent[0] - n[0] * dot_nt,
                    tangent[1] - n[1] * dot_nt,
                    tangent[2] - n[2] * dot_nt,
                ];

                // Normalize tangent
                let len_t =
                    (ortho_t[0] * ortho_t[0] + ortho_t[1] * ortho_t[1] + ortho_t[2] * ortho_t[2])
                        .sqrt();
                vertex.tangent = [ortho_t[0] / len_t, ortho_t[1] / len_t, ortho_t[2] / len_t];

                // Bitangent = cross(normal, tangent)
                vertex.bitangent = [
                    n[1] * vertex.tangent[2] - n[2] * vertex.tangent[1],
                    n[2] * vertex.tangent[0] - n[0] * vertex.tangent[2],
                    n[0] * vertex.tangent[1] - n[1] * vertex.tangent[0],
                ];
            });
            return;
        }

        // Initialize all tangents and bitangents to zero
        vertices.par_iter_mut().for_each(|vertex| {
            vertex.tangent = [0.0, 0.0, 0.0];
            vertex.bitangent = [0.0, 0.0, 0.0];
        });

        // Pre-calculate tangent/bitangent contributions per triangle
        let triangle_contributions: Vec<_> = (0..indices.len())
            .into_par_iter()
            .step_by(3)
            .map(|i| {
                let i0 = indices[i] as usize;
                let i1 = indices[i + 1] as usize;
                let i2 = indices[i + 2] as usize;

                let v0 = &vertices[i0];
                let v1 = &vertices[i1];
                let v2 = &vertices[i2];

                // Position deltas
                let edge1 = [
                    v1.position[0] - v0.position[0],
                    v1.position[1] - v0.position[1],
                    v1.position[2] - v0.position[2],
                ];
                let edge2 = [
                    v2.position[0] - v0.position[0],
                    v2.position[1] - v0.position[1],
                    v2.position[2] - v0.position[2],
                ];

                // UV deltas
                let delta_uv1 = [v1.tex_uv[0] - v0.tex_uv[0], v1.tex_uv[1] - v0.tex_uv[1]];
                let delta_uv2 = [v2.tex_uv[0] - v0.tex_uv[0], v2.tex_uv[1] - v0.tex_uv[1]];

                // Calculate tangent and bitangent
                let det = delta_uv1[0] * delta_uv2[1] - delta_uv1[1] * delta_uv2[0];
                let r = if det.abs() > 1e-6 { 1.0 / det } else { 0.0 };

                let tangent = [
                    r * (delta_uv2[1] * edge1[0] - delta_uv1[1] * edge2[0]),
                    r * (delta_uv2[1] * edge1[1] - delta_uv1[1] * edge2[1]),
                    r * (delta_uv2[1] * edge1[2] - delta_uv1[1] * edge2[2]),
                ];

                let bitangent = [
                    r * (-delta_uv2[0] * edge1[0] + delta_uv1[0] * edge2[0]),
                    r * (-delta_uv2[0] * edge1[1] + delta_uv1[0] * edge2[1]),
                    r * (-delta_uv2[0] * edge1[2] + delta_uv1[0] * edge2[2]),
                ];

                (i0, i1, i2, tangent, bitangent)
            })
            .collect();

        // Accumulate contributions (must be sequential due to race conditions)
        for (i0, i1, i2, tangent, bitangent) in triangle_contributions {
            vertices[i0].tangent[0] += tangent[0];
            vertices[i0].tangent[1] += tangent[1];
            vertices[i0].tangent[2] += tangent[2];

            vertices[i1].tangent[0] += tangent[0];
            vertices[i1].tangent[1] += tangent[1];
            vertices[i1].tangent[2] += tangent[2];

            vertices[i2].tangent[0] += tangent[0];
            vertices[i2].tangent[1] += tangent[1];
            vertices[i2].tangent[2] += tangent[2];

            vertices[i0].bitangent[0] += bitangent[0];
            vertices[i0].bitangent[1] += bitangent[1];
            vertices[i0].bitangent[2] += bitangent[2];

            vertices[i1].bitangent[0] += bitangent[0];
            vertices[i1].bitangent[1] += bitangent[1];
            vertices[i1].bitangent[2] += bitangent[2];

            vertices[i2].bitangent[0] += bitangent[0];
            vertices[i2].bitangent[1] += bitangent[1];
            vertices[i2].bitangent[2] += bitangent[2];
        }

        // Normalize and orthogonalize in parallel
        vertices.par_iter_mut().for_each(|vertex| {
            let n = vertex.normal;
            let t = vertex.tangent;

            // Gram-Schmidt orthogonalize
            let dot_nt = n[0] * t[0] + n[1] * t[1] + n[2] * t[2];

            let ortho_t = [
                t[0] - n[0] * dot_nt,
                t[1] - n[1] * dot_nt,
                t[2] - n[2] * dot_nt,
            ];

            // Normalize tangent
            let len_t =
                (ortho_t[0] * ortho_t[0] + ortho_t[1] * ortho_t[1] + ortho_t[2] * ortho_t[2])
                    .sqrt();
            if len_t > 1e-6 {
                vertex.tangent = [ortho_t[0] / len_t, ortho_t[1] / len_t, ortho_t[2] / len_t];
            } else {
                // Fallback for degenerate cases
                if n[0].abs() > 0.9 {
                    vertex.tangent = [0.0, 1.0, 0.0];
                } else {
                    vertex.tangent = [1.0, 0.0, 0.0];
                }
            }

            // Normalize bitangent
            let b = vertex.bitangent;
            let len_b = (b[0] * b[0] + b[1] * b[1] + b[2] * b[2]).sqrt();
            if len_b > 1e-6 {
                vertex.bitangent = [b[0] / len_b, b[1] / len_b, b[2] / len_b];
            } else {
                // Calculate bitangent from cross product
                vertex.bitangent = [
                    n[1] * vertex.tangent[2] - n[2] * vertex.tangent[1],
                    n[2] * vertex.tangent[0] - n[0] * vertex.tangent[2],
                    n[0] * vertex.tangent[1] - n[1] * vertex.tangent[0],
                ];
            }
        });
    }

    /// Loads a primitive mesh from embedded .glb bytes and returns its vertex and index data
    fn load_primitive_from_glb(bytes: &'static [u8]) -> (Vec<Vertex>, Vec<u32>) {
        let gltf = gltf::import_slice(bytes).expect("Failed to load primitive from embedded bytes");
        let (doc, buffers, _images) = gltf;

        let gltf_scene = doc
            .default_scene()
            .or_else(|| doc.scenes().next())
            .expect("Primitive glb has no scene");

        // Get the first mesh's first primitive
        let node = gltf_scene
            .nodes()
            .next()
            .expect("No nodes in primitive glb");
        let mesh = node.mesh().expect("Node has no mesh");
        let primitive = mesh.primitives().next().expect("Mesh has no primitives");

        let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

        let positions: Vec<[f32; 3]> = reader
            .read_positions()
            .expect("Primitive has no positions")
            .collect();

        let normals: Vec<[f32; 3]> = reader.read_normals().map_or_else(
            || vec![[0.0, 0.0, 1.0]; positions.len()],
            |iter| iter.collect(),
        );

        let tex_coords: Vec<[f32; 2]> = reader.read_tex_coords(0).map_or_else(
            || vec![[0.0, 0.0]; positions.len()],
            |coords| coords.into_f32().collect(),
        );

        let tangents: Vec<[f32; 4]> = reader
            .read_tangents()
            .map_or_else(Vec::new, |iter| iter.collect());

        let indices: Vec<u32> = reader
            .read_indices()
            .map_or_else(Vec::new, |iter| iter.into_u32().collect());

        // Build vertices
        let mut vertices: Vec<Vertex> = if !tangents.is_empty() {
            use glam::Vec3;
            positions
                .into_iter()
                .enumerate()
                .map(|(i, pos)| {
                    let tangent_vec3: Vec3 =
                        [tangents[i][0], tangents[i][1], tangents[i][2]].into();
                    let handedness = tangents[i][3];
                    let normal: Vec3 = normals[i].into();

                    let bitangent = normal.cross(tangent_vec3) * handedness;
                    Vertex {
                        position: pos,
                        normal: normal.into(),
                        tex_uv: tex_coords[i],
                        tangent: tangent_vec3.into(),
                        bitangent: bitangent.into(),
                    }
                })
                .collect()
        } else {
            // No tangents provided, create vertices without them
            positions
                .into_iter()
                .enumerate()
                .map(|(i, pos)| Vertex {
                    position: pos,
                    normal: normals[i],
                    tex_uv: tex_coords[i],
                    tangent: [0.0, 0.0, 0.0],
                    bitangent: [0.0, 0.0, 0.0],
                })
                .collect()
        };

        // Calculate tangents if not provided
        if tangents.is_empty() {
            Self::calculate_tangents(&mut vertices, &indices);
        }

        (vertices, indices)
    }

    /// Gets or initializes a primitive mesh data from embedded .glb bytes
    fn get_primitive(
        primitive: &'static OnceLock<PrimitiveMeshData>,
        bytes: &'static [u8],
    ) -> &'static PrimitiveMeshData {
        primitive.get_or_init(|| {
            let (vertices, indices) = Self::load_primitive_from_glb(bytes);
            PrimitiveMeshData {
                vertex_buffer: RenderContext::create_vertex_buffer_lazy(&vertices),
                index_buffer: RenderContext::create_index_buffer_lazy(&indices),
            }
        })
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
            vertex_buffer: None,
            index_buffer: None,
        }
    }
}

pub struct Mesh3DBuilder {
    proto: NodePrototype,
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    material: MaterialProperties,
    // Optional pre-existing buffers for primitives (cloning is cheap since LazyBuffer uses Arc)
    vertex_buffer: Option<LazyBuffer<[Vertex]>>,
    index_buffer: Option<LazyBuffer<[u32]>>,
}

impl Builder for Mesh3DBuilder {
    type Node = Mesh3D;

    fn prototype(&mut self) -> &mut NodePrototype {
        &mut self.proto
    }

    fn build(self) -> Self::Node {
        let default_data = Mesh3DUniformBufferData::default();

        // Use pre-existing buffers if available, otherwise create from vertices/indices
        let vertex_buffer = self
            .vertex_buffer
            .unwrap_or_else(|| RenderContext::create_vertex_buffer_lazy(&self.vertices));
        let index_buffer = self
            .index_buffer
            .unwrap_or_else(|| RenderContext::create_index_buffer_lazy(&self.indices));

        Mesh3D {
            transform: self.proto.transform,
            children: self.proto.children,
            events: self.proto.events,
            vertex_buffer,
            index_buffer,
            material: self.material,

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
            vertex_buffer: None,
            index_buffer: None,
        }
    }
}
