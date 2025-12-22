//! Model node that can be used to load 3D models from GLTF/GLB files or primitive shapes.
//!
//! # Usage
//! add the Model to the scene tree using the Scene and the engine will render the model where its defined given you have a camera and shader defined.
//!
//! # Example
//! ```rust,no_run
//! use maple::nodes::{Buildable, Builder, Model, model::Primitive};
//! use std::path::Path;
//!
//! Model::builder()
//!     .add_primitive(Primitive::Cube) // add a primitive mesh to the model
//!     .load_gltf(Path::new("res/models/scene.glb")) // load a glb or gltf model
//!     .cast_shadows(true)
//!     .has_lighting(true)
//!     .build();
//! ```

use std::{
    collections::HashMap,
    io::Write,
    path::Path,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use crate::components::{material::MaterialProperties, mesh::Mesh};
use glam::{self as math, Quat};
use gltf::{Document, material::AlphaMode};
use maple_engine::{
    Buildable, Builder, Node, Scene, components::NodeTransform, nodes::node_builder::NodePrototype,
    prelude::EventReceiver,
};
use maple_renderer::core::texture::{Texture, TextureFormat};
use math::{Vec2, Vec3, Vec4};

/// Primitive shapes that can be loaded
pub enum Primitive {
    /// Cube primitive
    Cube,
    /// Sphere primitive
    Sphere,
    /// Smooth shaded Sphere primitive
    SmoothSphere,
    /// Plane primitive
    Plane,
    /// Pyramid primitive
    Pyramid,
    /// Cylinder primitive
    Cylinder,
    /// Torus primitive
    Torus,
    /// Cone primitive
    Cone,
    /// Teapot primitive
    Teapot,
}

/// Vertex of a mesh
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    /// position of the vertex
    pub position: math::Vec3,
    /// normal of the vertex
    pub normal: math::Vec3,
    /// color of the vertex
    pub color: math::Vec4,
    /// texture uv of the vertex
    pub tex_uv: math::Vec2,
    /// tangent of the vertex for normal mapping
    pub tangent: math::Vec3,
    /// bitangent of the vertex for normal mapping
    pub bitangent: math::Vec3,
}

/// Mesh node that holds the mesh data
#[derive(Clone, Debug)]
pub struct MeshNode {
    /// name of the node
    _name: String,
    /// relative transformation of the node
    pub transform: NodeTransform,
    /// mesh primitives of the node
    mesh_primitives: Vec<Mesh>,
}

/// Model node that holds the mesh nodes from a file or primitive shapes
#[derive(Clone)]
pub struct Model {
    /// mesh nodes of the model
    pub nodes: Vec<MeshNode>,
    /// transformation of the model
    pub transform: NodeTransform,
    /// children of the model
    pub children: Scene,

    events: EventReceiver,

    cast_shadows: bool,

    has_lighting: bool,
}

impl Node for Model {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }

    fn get_children(&self) -> &Scene {
        &self.children
    }

    fn get_children_mut(&mut self) -> &mut Scene {
        &mut self.children
    }

    fn get_events(&mut self) -> &mut EventReceiver {
        &mut self.events
    }
}

impl Model {
    /// load a primitive shape model the shapes are self explanatory
    ///
    /// # Arguments
    /// - `primitive` - the primitive shape to load
    ///
    /// # Returns
    /// the model node with the primitive shape loaded
    pub fn new_primitive(primitive: Primitive) -> Model {
        let nodes = match primitive {
            Primitive::Cube => {
                self::Model::from_slice(include_bytes!("../../res/primitives/cube.glb"))
            }
            Primitive::Sphere => {
                self::Model::from_slice(include_bytes!("../../res/primitives/sphere.glb"))
            }
            Primitive::SmoothSphere => {
                self::Model::from_slice(include_bytes!("../../res/primitives/smooth_sphere.glb"))
            }
            Primitive::Plane => {
                self::Model::from_slice(include_bytes!("../../res/primitives/plane.glb"))
            }
            Primitive::Pyramid => {
                self::Model::from_slice(include_bytes!("../../res/primitives/pyramid.glb"))
            }
            Primitive::Torus => {
                self::Model::from_slice(include_bytes!("../../res/primitives/torus.glb"))
            }
            Primitive::Cylinder => {
                self::Model::from_slice(include_bytes!("../../res/primitives/cylinder.glb"))
            }
            Primitive::Cone => {
                self::Model::from_slice(include_bytes!("../../res/primitives/cone.glb"))
            }
            Primitive::Teapot => {
                self::Model::from_slice(include_bytes!("../../res/primitives/teapot.glb"))
            }
        };

        Model {
            nodes,
            cast_shadows: true,
            has_lighting: true,
            transform: NodeTransform::default(),
            children: Scene::new(),
            events: EventReceiver::default(),
        }
    }

    /// load a model from a gltf file
    ///
    /// # Arguments
    /// * `file` - the path to the gltf file
    ///
    /// # Returns
    /// the model node with the model loaded
    ///
    /// # Panics
    /// if the file does not exist or is not a valid gltf file
    pub fn new_gltf(file: &str) -> Model {
        let model_loaded = Arc::new(AtomicBool::new(false));
        let model_loaded_clone = model_loaded.clone();
        let loading_thread = thread::spawn(move || {
            let animation = ["\\", "|", "/", "-"];
            let mut i = 0;
            while !model_loaded_clone.load(Ordering::SeqCst) {
                print!("{}", format!("\rloading model: {}", animation[i]).cyan()); // Overwrite the previous line
                std::io::stdout().flush().unwrap();
                i = (i + 1) % 4;

                thread::sleep(Duration::from_millis(50));
            }

            // clear the loading animation
            print!("\r                                \r");
            std::io::stdout().flush().unwrap();
        });

        let gltf = gltf::import(Path::new(file)).expect("failed to open GLTF file");

        println!(
            "gltf file declared these unsupported extensions: {:?}",
            gltf.0.extensions_used()
        );
        println!(
            "gltf file requires these unsupported extensions: {:?}",
            gltf.0.extensions_required()
        );

        //end thread here
        model_loaded.store(true, Ordering::SeqCst);
        loading_thread.join().unwrap();

        let nodes = Self::build_model(gltf);

        Model {
            nodes,
            cast_shadows: true,
            has_lighting: true,
            transform: NodeTransform::default(),
            children: Scene::new(),
            events: EventReceiver::default(),
        }
    }

    fn from_slice(data: &[u8]) -> Vec<MeshNode> {
        let gltf = gltf::import_slice(data).expect("failed to open GLTF file");

        Self::build_model(gltf)
    }

    fn build_model(
        gltf: (Document, Vec<gltf::buffer::Data>, Vec<gltf::image::Data>),
    ) -> Vec<MeshNode> {
        let (doc, buffers, images) = gltf;
        let mut nodes: Vec<MeshNode> = Vec::new();

        let mut texture_cache: HashMap<usize, Rc<Texture>> = HashMap::new(); // Cache with key as image index and value as a smart pointer to the texture

        for node in doc.nodes() {
            let (translation, rotation, scale) = node.transform().decomposed();

            let translation: Vec3 = translation.into();
            let rotation: Quat = Quat::from_array(rotation);
            let scale: Vec3 = scale.into();

            if let Some(mesh) = node.mesh() {
                let mut primitive_meshes: Vec<Mesh> = Vec::new();

                for primitive in mesh.primitives() {
                    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                    // Get vertex data from reader
                    let positions: Vec<[f32; 3]> = reader
                        .read_positions()
                        .map_or_else(Vec::new, |iter| iter.collect());

                    let normals: Vec<[f32; 3]> = reader.read_normals().map_or_else(
                        || vec![[0.0, 0.0, 1.0]; positions.len()],
                        |iter| iter.collect(),
                    );

                    let tex_coords: Vec<[f32; 2]> = reader.read_tex_coords(0).map_or_else(
                        || vec![[0.0, 0.0]; positions.len()],
                        |coords| coords.into_f32().collect(),
                    );

                    let color =
                        reader
                            .read_colors(0)
                            .map_or(math::vec4(1.0, 1.0, 1.0, 1.0), |colors| {
                                Vec4::from_array(
                                    colors
                                        .into_rgba_f32()
                                        .next()
                                        .unwrap_or([1.0, 1.0, 1.0, 1.0]),
                                )
                            });

                    let indices: Vec<u32> = reader
                        .read_indices()
                        .map_or_else(Vec::new, |iter| iter.into_u32().collect());

                    // Construct vertices from the extracted data
                    let mut vertices: Vec<Vertex> = positions
                        .into_iter()
                        .enumerate()
                        .map(|(i, pos)| Vertex {
                            position: pos.into(),
                            normal: normals[i].into(),
                            tex_uv: tex_coords[i].into(),
                            color,
                            tangent: math::Vec3::ZERO,
                            bitangent: math::Vec3::ZERO,
                        })
                        .collect();

                    // calculate_tangents
                    Self::calculate_tangents(&mut vertices, &indices);

                    let base_color_texture = Self::load_texture(
                        &primitive,
                        |m| {
                            m.pbr_metallic_roughness()
                                .base_color_texture()
                                .map(|t| t.texture().source().index())
                        },
                        &mut texture_cache,
                        &images,
                    );

                    let metallic_roughness_texture = Self::load_texture(
                        &primitive,
                        |m| {
                            m.pbr_metallic_roughness()
                                .metallic_roughness_texture()
                                .map(|t| t.texture().source().index())
                        },
                        &mut texture_cache,
                        &images,
                    );

                    let normal_texture = Self::load_texture(
                        &primitive,
                        |m| m.normal_texture().map(|t| t.texture().source().index()),
                        &mut texture_cache,
                        &images,
                    );

                    let occlusion_texture = Self::load_texture(
                        &primitive,
                        |m| m.occlusion_texture().map(|f| f.texture().source().index()),
                        &mut texture_cache,
                        &images,
                    );

                    let emissive_texture = Self::load_texture(
                        &primitive,
                        |m| m.emissive_texture().map(|t| t.texture().source().index()),
                        &mut texture_cache,
                        &images,
                    );

                    // Create the mesh
                    let mesh = Mesh::new(
                        vertices,
                        indices,
                        MaterialProperties {
                            base_color_factor: Vec4::from(
                                &primitive
                                    .material()
                                    .pbr_metallic_roughness()
                                    .base_color_factor(),
                            ),
                            base_color_texture,

                            metallic_factor: primitive
                                .material()
                                .pbr_metallic_roughness()
                                .metallic_factor(),
                            roughness_factor: primitive
                                .material()
                                .pbr_metallic_roughness()
                                .roughness_factor(),
                            metallic_roughness_texture,

                            normal_scale: primitive
                                .material()
                                .normal_texture()
                                .map(|m| m.scale())
                                .unwrap_or(1.0),
                            normal_texture,

                            ambient_occlusion_strength: primitive
                                .material()
                                .occlusion_texture()
                                .map(|m| m.strength())
                                .unwrap_or(1.0),
                            occlusion_texture,

                            emissive_factor: math::Vec3::from_column_slice(
                                primitive.material().emissive_factor().as_slice(),
                            ),
                            emissive_texture,

                            double_sided: primitive.material().double_sided(),
                            alpha_mode: match primitive.material().alpha_mode() {
                                gltf::material::AlphaMode::Opaque => AlphaMode::Opaque,
                                gltf::material::AlphaMode::Mask => AlphaMode::Mask,
                                gltf::material::AlphaMode::Blend => AlphaMode::Blend,
                            },
                            alpha_cutoff: primitive.material().alpha_cutoff().unwrap_or(0.5),
                        },
                    );
                    primitive_meshes.push(mesh);
                }

                let transform = NodeTransform::new(translation, rotation, scale);

                let node = MeshNode {
                    _name: node.name().unwrap_or_default().to_string(),
                    transform,
                    mesh_primitives: primitive_meshes,
                };
                nodes.push(node);
            }
        }

        nodes
    }

    fn load_texture<'a>(
        primitive: &gltf::Primitive<'a>,
        index_fn: impl Fn(&gltf::Material<'a>) -> Option<usize>,
        texture_cache: &mut HashMap<usize, Rc<Texture>>,
        image: &[gltf::image::Data],
    ) -> Option<Rc<Texture>> {
        if let Some(image_index) = index_fn(&primitive.material()) {
            let shared_texture = texture_cache
                .entry(image_index)
                .or_insert_with(|| {
                    let image = &image[image_index];

                    let format = match image.format {
                        gltf::image::Format::R8G8B8A8 => TextureFormat::RGBA8,
                        gltf::image::Format::R8G8B8 => TextureFormat::RGB8,
                        gltf::image::Format::R8 => TextureFormat::R8,
                        _ => panic!("unsupported image format not rgba, rgb, or r"),
                    };
                    Rc::new(Texture::load_from_gltf(
                        &image.pixels,
                        image.width,
                        image.height,
                        format,
                    ))
                })
                .clone();
            return Some(shared_texture);
        }
        None
    }

    /// calculates the tangent and bitangent for each triangle of the mesh
    fn calculate_tangents(vertices: &mut Vec<Vertex>, indices: &[u32]) {
        if !indices.is_empty() {
            for triangle in indices.chunks(3) {
                let i0 = triangle[0] as usize;
                let i1 = triangle[1] as usize;
                let i2 = triangle[2] as usize;

                let v0 = &vertices[i0];
                let v1 = &vertices[i1];
                let v2 = &vertices[i2];

                let edge1 = v1.position - v0.position;
                let edge2 = v2.position - v0.position;

                let delta_uv1 = v1.tex_uv - v0.tex_uv;
                let delta_uv2 = v2.tex_uv - v0.tex_uv;

                let f = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv2.x * delta_uv1.y);

                let tangent = f * Vec3::new(
                    delta_uv2.y * edge1.x - delta_uv1.y * edge2.x,
                    delta_uv2.y * edge1.y - delta_uv1.y * edge2.y,
                    delta_uv2.y * edge1.z - delta_uv1.y * edge2.z,
                );

                let bitangent = f * Vec3::new(
                    -delta_uv2.x * edge1.x + delta_uv1.x * edge2.x,
                    -delta_uv2.x * edge1.y + delta_uv1.x * edge2.y,
                    -delta_uv2.x * edge1.z + delta_uv1.x * edge2.z,
                );

                vertices[i0].tangent += tangent;
                vertices[i1].tangent += tangent;
                vertices[i2].tangent += tangent;

                vertices[i0].bitangent += bitangent;
                vertices[i1].bitangent += bitangent;
                vertices[i2].bitangent += bitangent;
            }
        }

        // finally normalize them
        for v in vertices {
            v.tangent = v.tangent.normalize();
            v.bitangent = v.bitangent.normalize();
        }
    }

    /// configures if this model casts shadows
    pub fn casts_shadows(&mut self, cast_shadow: bool) -> &mut Self {
        self.cast_shadows = cast_shadow;
        self
    }

    /// configures if this model is affected by lights
    pub fn has_lighting(&mut self, lighting: bool) -> &mut Self {
        self.has_lighting = lighting;
        self
    }

    /// set the material of every mesh within the model
    pub fn set_material(&mut self, material: MaterialProperties) -> &mut Self {
        for node in &mut self.nodes {
            for mesh in &mut node.mesh_primitives {
                mesh.set_material(material.clone());
            }
        }
        self
    }
}

impl Buildable for Model {
    type Builder = ModelBuilder;

    fn builder() -> Self::Builder {
        ModelBuilder {
            prototype: NodePrototype::default(),
            has_lighting: true,
            cast_shadows: true,
            nodes: Vec::new(),
        }
    }
}

/// builder for the [`Model`]
pub struct ModelBuilder {
    prototype: NodePrototype,
    has_lighting: bool,
    cast_shadows: bool,
    nodes: Vec<MeshNode>,
}

impl ModelBuilder {
    /// load a model from a gltf file
    ///
    /// # Arguments
    /// * `file` - the path to the gltf file
    ///
    /// # Returns
    /// the model node with the model loaded
    ///
    /// # Panics
    /// if the file does not exist or is not a valid gltf file
    pub fn load_gltf(&mut self, file: &Path) -> &mut Self {
        let model_loaded = Arc::new(AtomicBool::new(false));
        let model_loaded_clone = model_loaded.clone();
        let loading_thread = thread::spawn(move || {
            let animation = ["\\", "|", "/", "-"];
            let mut i = 0;
            while !model_loaded_clone.load(Ordering::SeqCst) {
                print!("{}", format!("\rloading model: {}", animation[i])); // Overwrite the previous line
                std::io::stdout().flush().unwrap();
                i = (i + 1) % 4;

                thread::sleep(Duration::from_millis(50));
            }

            // clear the loading animation
            print!("\r                                \r");
            std::io::stdout().flush().unwrap();
        });

        let gltf = gltf::import(file).expect("failed to open GLTF file");

        //end thread here
        model_loaded.store(true, Ordering::SeqCst);
        loading_thread.join().unwrap();

        self.nodes.extend(Model::build_model(gltf));

        self
    }

    /// adds a primitive mesh to the model
    pub fn add_primitive(&mut self, primitive: Primitive) -> &mut Self {
        let nodes = match primitive {
            Primitive::Cube => {
                self::Model::from_slice(include_bytes!("../../res/primitives/cube.glb"))
            }
            Primitive::Sphere => {
                self::Model::from_slice(include_bytes!("../../res/primitives/sphere.glb"))
            }
            Primitive::SmoothSphere => {
                self::Model::from_slice(include_bytes!("../../res/primitives/smooth_sphere.glb"))
            }
            Primitive::Plane => {
                self::Model::from_slice(include_bytes!("../../res/primitives/plane.glb"))
            }
            Primitive::Pyramid => {
                self::Model::from_slice(include_bytes!("../../res/primitives/pyramid.glb"))
            }
            Primitive::Torus => {
                self::Model::from_slice(include_bytes!("../../res/primitives/torus.glb"))
            }
            Primitive::Cylinder => {
                self::Model::from_slice(include_bytes!("../../res/primitives/cylinder.glb"))
            }
            Primitive::Cone => {
                self::Model::from_slice(include_bytes!("../../res/primitives/cone.glb"))
            }
            Primitive::Teapot => {
                self::Model::from_slice(include_bytes!("../../res/primitives/teapot.glb"))
            }
        };

        self.nodes.extend(nodes);

        self
    }

    /// configures if the model is affected by lighting or not
    pub fn has_lighting(&mut self, lighting: bool) -> &mut Self {
        self.has_lighting = lighting;
        self
    }

    /// configures of the model casts shadows
    pub fn cast_shadows(&mut self, shadows: bool) -> &mut Self {
        self.cast_shadows = shadows;
        self
    }
}

impl Builder for ModelBuilder {
    type Node = Model;

    fn prototype(&mut self) -> &mut NodePrototype {
        &mut self.prototype
    }

    fn build(&mut self) -> Self::Node {
        let proto = self.prototype().take();
        Model {
            transform: proto.transform,
            events: proto.events,
            children: proto.children,
            has_lighting: self.has_lighting,
            cast_shadows: self.cast_shadows,
            nodes: std::mem::take(&mut self.nodes),
        }
    }
}

// pub trait ModelBuilder {
//     fn create_gltf(file: &str) -> NodeBuilder<Model> {
//         NodeBuilder::new(Model::new_gltf(file))
//     }
//     fn create_primitive(primitive: Primitive) -> NodeBuilder<Model> {
//         NodeBuilder::new(Model::new_primitive(primitive))
//     }
//     fn cast_shadows(&mut self, value: bool) -> &mut Self;
//     fn has_lighting(&mut self, value: bool) -> &mut Self;
//     fn set_material(&mut self, material: MaterialProperties) -> &mut Self;
//     //    fn set_material_base_color(&mut self, color: math::Vec4) -> &mut Self;
// }
//
// impl ModelBuilder for NodeBuilder<Model> {
//     fn cast_shadows(&mut self, value: bool) -> &mut Self {
//         self.node.casts_shadows(value);
//         self
//     }
//     fn has_lighting(&mut self, value: bool) -> &mut Self {
//         self.node.has_lighting(value);
//         self
//     }
//     fn set_material(&mut self, material: MaterialProperties) -> &mut Self {
//         self.node.set_material(material);
//         self
//     }
//
//     // fn set_material_base_color(&mut self, color: math::Vec4) -> &mut Self {
//     //     let material = MaterialProperties::new(color, 1.0, 1.0, false, AlphaMode::Opaque, 1.0);
//     //     self.set_material(material);
//     //     self
//     // }
// }
