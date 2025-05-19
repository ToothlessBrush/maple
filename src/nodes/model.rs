//! Model node that can be used to load 3D models from GLTF/GLB files or primitive shapes.
//!
//! # Usage
//! add the Model to the scene tree using the Scene and the engine will render the model where its defined given you have a camera and shader defined.
//!
//! ```rust
//! use maple::game_context::nodes::model::Model;
//! use maple::game_context::nodes::model::Primitive;
//! use maple::game_context::GameContext;
//! use maple::Engine;
//! use nalgebra_glm as math;
//!
//! let mut engine = Engine::init("example", 800, 600);
//!
//! engine.context.nodes.add("model", Model::new_primitive(Primitive::Cube));
//!
//! // or load a model
//!
//! //engine.context.nodes.add("model", Model::new_gltf("res/models/model.gltf"));
//!
//! //engine.begin();
//! ```

use crate::components::node_transform::WorldTransform;
use crate::gl;

use gltf::Document;
use math::Vec3;
use nalgebra_glm as math;
use std::io::Write;
use std::{collections::HashMap, path::Path, rc::Rc};

use colored::*;

use std::thread;
use std::time::Duration;

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::renderer::texture::TextureType;
use crate::renderer::{shader::Shader, texture::Texture};

use crate::components::{EventReceiver, NodeTransform};

use crate::components::{
    Mesh,
    mesh::{AlphaMode, MaterialProperties},
};

use super::Node;
use super::NodeBuilder;
use super::camera::Camera3D;
use super::node::Drawable;
use crate::context::scene::Scene;

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

impl Drawable for Model {
    fn draw(&self, shader: &mut Shader, camera: &Camera3D) {
        shader.bind();
        shader.set_uniform("u_LightingEnabled", self.has_lighting);

        //draw order
        // 1. opaque meshes
        // 2. transparent meshes sorted by distance from camera
        let camera_position = camera.transform.world_space().position;

        let mut opaque_meshes: Vec<(&Mesh, WorldTransform)> = Vec::new();
        let mut transparent_meshes: Vec<(&Mesh, WorldTransform)> = Vec::new();

        let parent_transform = self.transform.world_space();

        for node in &self.nodes {
            // add the mesh nodes transform to the models transform to get the world position
            let world_relative = *parent_transform + node.transform.into();
            for mesh in &node.mesh_primitives {
                match mesh.material_properties.alpha_mode {
                    AlphaMode::Opaque => {
                        opaque_meshes.push((mesh, world_relative));
                    }
                    AlphaMode::Blend | AlphaMode::Mask => {
                        transparent_meshes.push((mesh, world_relative));
                    }
                }
            }
        }

        shader.bind();
        shader.set_uniform("u_VP", camera.get_vp_matrix());

        // Draw all opaque meshes first
        for (mesh, transform) in &mut opaque_meshes {
            // println!("{:?}", transform);
            shader.set_uniform("u_Model", transform.matrix);

            mesh.draw(shader, camera);
        }

        // Sort transparent meshes by distance (back-to-front)
        transparent_meshes.sort_by(|a, b| {
            let a_distance = math::length(&(camera_position - a.1.position)) as i32;
            let b_distance = math::length(&(camera_position - b.1.position)) as i32;
            b_distance.cmp(&a_distance)
        });

        // Draw transparent meshes in sorted order
        for (mesh, transform) in &mut transparent_meshes {
            shader.set_uniform("u_Model", transform.matrix);
            mesh.draw(shader, camera);
        }
    }

    fn draw_shadow(&self, depth_shader: &mut Shader) {
        if !self.cast_shadows {
            return;
        }

        let parent_transform = self.transform.world_space();

        for node in &self.nodes {
            // add the mesh nodes transform to the models transform to get the world position
            let world_relative = *parent_transform + node.transform.into();
            depth_shader.bind();
            depth_shader.set_uniform("u_Model", world_relative.matrix);

            for mesh in &node.mesh_primitives {
                mesh.draw_shadow(depth_shader);
            }
        }
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
        match primitive {
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

        //end thread here
        model_loaded.store(true, Ordering::SeqCst);
        loading_thread.join().unwrap();

        Self::build_model(gltf)
    }

    fn from_slice(data: &[u8]) -> Model {
        let gltf = gltf::import_slice(data).expect("failed to open GLTF file");

        Self::build_model(gltf)
    }

    fn build_model(gltf: (Document, Vec<gltf::buffer::Data>, Vec<gltf::image::Data>)) -> Model {
        let (doc, buffers, images) = gltf;
        let mut nodes: Vec<MeshNode> = Vec::new();

        let mut texture_cache: HashMap<usize, Rc<Texture>> = HashMap::new(); // Cache with key as image index and value as a smart pointer to the texture

        for node in doc.nodes() {
            let (translation, rotation, scale) = node.transform().decomposed();

            let translation: Vec3 = math::make_vec3(&translation);
            let rotation = math::make_quat(&rotation);
            let scale: Vec3 = math::make_vec3(&scale);

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
                                math::make_vec4(
                                    &colors
                                        .into_rgba_f32()
                                        .next()
                                        .unwrap_or([1.0, 1.0, 1.0, 1.0]),
                                )
                            });

                    let indices: Vec<u32> = reader
                        .read_indices()
                        .map_or_else(Vec::new, |iter| iter.into_u32().collect());

                    // Construct vertices from the extracted data
                    let vertices: Vec<Vertex> = positions
                        .into_iter()
                        .enumerate()
                        .map(|(i, pos)| Vertex {
                            position: math::make_vec3(&pos),
                            normal: math::make_vec3(&normals[i]),
                            tex_uv: math::make_vec2(&tex_coords[i]),
                            color,
                        })
                        .collect();

                    let base_color_texture = Self::load_texture(
                        &primitive,
                        |m| {
                            m.pbr_metallic_roughness()
                                .base_color_texture()
                                .map(|t| t.texture().source().index())
                        },
                        &mut texture_cache,
                        &images,
                        TextureType::BaseColor,
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
                        TextureType::MetallicRoughness,
                    );

                    let normal_texture = Self::load_texture(
                        &primitive,
                        |m| m.normal_texture().map(|t| t.texture().source().index()),
                        &mut texture_cache,
                        &images,
                        TextureType::NormalMap,
                    );

                    let occlusion_texture = Self::load_texture(
                        &primitive,
                        |m| m.occlusion_texture().map(|f| f.texture().source().index()),
                        &mut texture_cache,
                        &images,
                        TextureType::Occlusion,
                    );

                    let emissive_texture = Self::load_texture(
                        &primitive,
                        |m| m.emissive_texture().map(|t| t.texture().source().index()),
                        &mut texture_cache,
                        &images,
                        TextureType::Emissive,
                    );

                    // Create the mesh
                    let mesh = Mesh::new(
                        vertices,
                        indices,
                        MaterialProperties {
                            base_color_factor: math::make_vec4(
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

        Model {
            nodes,
            cast_shadows: true,
            has_lighting: true,
            transform: NodeTransform::default(),
            children: Scene::new(),
            events: EventReceiver::default(),
        }
    }

    fn load_texture<'a>(
        primitive: &gltf::Primitive<'a>,
        index_fn: impl Fn(&gltf::Material<'a>) -> Option<usize>,
        texture_cache: &mut HashMap<usize, Rc<Texture>>,
        image: &[gltf::image::Data],
        texture_type: TextureType,
    ) -> Option<Rc<Texture>> {
        if let Some(image_index) = index_fn(&primitive.material()) {
            let shared_texture = texture_cache
                .entry(image_index)
                .or_insert_with(|| {
                    let image = &image[image_index];

                    println!("{:?}", image.format);
                    println!("{:?}", image.width);
                    println!("{:?}", image.height);

                    let format = match image.format {
                        gltf::image::Format::R8G8B8A8 => gl::RGBA,
                        gltf::image::Format::R8G8B8 => gl::RGB,
                        gltf::image::Format::R8 => gl::RED,
                        _ => panic!("unsupported image format not rgba, rgb, or r"),
                    };
                    Rc::new(Texture::load_from_gltf(
                        &image.pixels,
                        image.width,
                        image.height,
                        texture_type,
                        format,
                    ))
                })
                .clone();
            return Some(shared_texture);
        }
        None
    }

    pub fn casts_shadows(&mut self, cast_shadow: bool) -> &mut Self {
        self.cast_shadows = cast_shadow;
        self
    }

    pub fn has_lighting(&mut self, lighting: bool) -> &mut Self {
        self.has_lighting = lighting;
        self
    }

    pub fn set_material(&mut self, material: MaterialProperties) -> &mut Self {
        for node in &mut self.nodes {
            for mesh in &mut node.mesh_primitives {
                mesh.set_material(material.clone());
            }
        }
        self
    }

    pub fn shade_smooth(&mut self) {
        for node in &mut self.nodes {
            for primitive in &mut node.mesh_primitives {
                primitive.shade_smooth();
            }
        }
    }
}

pub trait ModelBuilder {
    fn create_gltf(file: &str) -> NodeBuilder<Model> {
        NodeBuilder::new(Model::new_gltf(file))
    }
    fn create_primitive(primitive: Primitive) -> NodeBuilder<Model> {
        NodeBuilder::new(Model::new_primitive(primitive))
    }
    fn cast_shadows(&mut self, value: bool) -> &mut Self;
    fn has_lighting(&mut self, value: bool) -> &mut Self;
    fn set_material(&mut self, material: MaterialProperties) -> &mut Self;
    //    fn set_material_base_color(&mut self, color: math::Vec4) -> &mut Self;
}

impl ModelBuilder for NodeBuilder<Model> {
    fn cast_shadows(&mut self, value: bool) -> &mut Self {
        self.node.casts_shadows(value);
        self
    }
    fn has_lighting(&mut self, value: bool) -> &mut Self {
        self.node.has_lighting(value);
        self
    }
    fn set_material(&mut self, material: MaterialProperties) -> &mut Self {
        self.node.set_material(material);
        self
    }

    // fn set_material_base_color(&mut self, color: math::Vec4) -> &mut Self {
    //     let material = MaterialProperties::new(color, 1.0, 1.0, false, AlphaMode::Opaque, 1.0);
    //     self.set_material(material);
    //     self
    // }
}
