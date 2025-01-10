//! Model node that can be used to load 3D models from GLTF/GLB files or primitive shapes.
//!
//! # Usage
//! add the Model to the scene tree using the NodeManager and the engine will render the model where its defined given you have a camera and shader defined.
//!
//! ```rust
//! use quaturn::game_context::nodes::model::Model;
//! use quaturn::game_context::nodes::model::Primitive;
//! use quaturn::game_context::GameContext;
//! use quaturn::Engine;
//! use nalgebra_glm as glm;
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

use glm::{Mat4, Vec3};
use gltf::Document;
use nalgebra_glm as glm;
use std::io::Write;
use std::{collections::HashMap, path::Path, rc::Rc};

use colored::*;

use std::thread;
use std::time::Duration;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::context::GameContext;

use crate::renderer::texture::TextureType;
use crate::renderer::{shader::Shader, texture::Texture};

use super::super::{Behavior, Drawable, Node, NodeManager, NodeTransform, Ready};
use super::mesh::AlphaMode;
use super::{camera::Camera3D, mesh, mesh::Mesh};

/// Primitive shapes that can be loaded
pub enum Primitive {
    /// Cube primitive
    Cube,
    /// Sphere primitive
    Sphere,
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
#[derive(Debug, Clone)]
#[repr(C)]
pub struct Vertex {
    /// position of the vertex
    pub position: glm::Vec3,
    /// normal of the vertex
    pub normal: glm::Vec3,
    /// color of the vertex
    pub color: glm::Vec4,
    /// texture uv of the vertex
    pub tex_uv: glm::Vec2,
}

/// Mesh node that holds the mesh data
pub struct MeshNode {
    /// name of the node
    _name: String,
    /// relative transformation of the node
    pub transform: NodeTransform,
    /// mesh primitives of the node
    mesh_primitives: Vec<Mesh>,
}

/// Model node that holds the mesh nodes from a file or primitive shapes
pub struct Model {
    /// mesh nodes of the model
    pub nodes: Vec<MeshNode>,
    /// transformation of the model
    pub transform: NodeTransform,
    /// children of the model
    pub children: NodeManager,
    /// callback to be called when the model is ready
    ready_callback: Option<Box<dyn FnMut(&mut Self)>>,
    /// callback to be called when the model is behaving
    behavior_callback: Option<Box<dyn FnMut(&mut Self, &mut GameContext)>>,
}

impl Node for Model {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }

    fn get_children(&mut self) -> &mut NodeManager {
        &mut self.children
    }

    fn as_ready(&mut self) -> Option<&mut (dyn Ready + 'static)> {
        Some(self)
    }

    fn as_behavior(&mut self) -> Option<&mut (dyn Behavior + 'static)> {
        Some(self)
    }
}

impl Ready for Model {
    fn ready(&mut self) {
        if let Some(mut callback) = self.ready_callback.take() {
            callback(self);
            self.ready_callback = Some(callback);
        }
    }
}

impl Behavior for Model {
    fn behavior(&mut self, context: &mut GameContext) {
        if let Some(mut callback) = self.behavior_callback.take() {
            callback(self, context);
            self.behavior_callback = Some(callback);
        }
    }
}

impl Drawable for Model {
    fn draw(&mut self, shader: &mut Shader, camera: &Camera3D) {
        //draw order
        // 1. opaque meshes
        // 2. transparent meshes sorted by distance from camera
        let camera_position = camera.get_position();

        let mut opaque_meshes: Vec<(&mut Mesh, NodeTransform)> = Vec::new();
        let mut transparent_meshes: Vec<(&mut Mesh, NodeTransform)> = Vec::new();

        for node in &mut self.nodes {
            for mesh in &mut node.mesh_primitives {
                match mesh.material_properties.alpha_mode {
                    AlphaMode::Opaque => {
                        opaque_meshes.push((mesh, node.transform.clone()));
                    }
                    AlphaMode::Blend | AlphaMode::Mask => {
                        transparent_meshes.push((mesh, node.transform.clone()));
                    }
                }
            }
        }

        // Draw all opaque meshes first
        for (mesh, transform) in &mut opaque_meshes {
            shader.bind();
            shader.set_uniform("u_Model", transform.matrix);
            mesh.draw(shader, camera);
        }

        // Sort transparent meshes by distance (back-to-front)
        transparent_meshes.sort_by(|a, b| {
            let a_distance = glm::length(&(camera_position - a.1.get_position())) as i32;
            let b_distance = glm::length(&(camera_position - b.1.get_position())) as i32;
            b_distance.cmp(&a_distance)
        });

        // Draw transparent meshes in sorted order
        for (mesh, transform) in &mut transparent_meshes {
            shader.bind();
            shader.set_uniform("u_Model", transform.matrix);
            mesh.draw(shader, camera);
        }
    }

    fn draw_shadow(&mut self, depth_shader: &mut Shader) {
        for node in &self.nodes {
            depth_shader.bind();
            depth_shader.set_uniform("u_Model", node.transform.matrix);

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
                self::Model::from_slice(include_bytes!("../../../../res/primitives/cube.glb"))
            }
            Primitive::Sphere => {
                self::Model::from_slice(include_bytes!("../../../../res/primitives/sphere.glb"))
            }
            Primitive::Plane => {
                self::Model::from_slice(include_bytes!("../../../../res/primitives/plane.glb"))
            }
            Primitive::Pyramid => {
                self::Model::from_slice(include_bytes!("../../../../res/primitives/pyramid.glb"))
            }
            Primitive::Torus => {
                self::Model::from_slice(include_bytes!("../../../../res/primitives/torus.glb"))
            }
            Primitive::Cylinder => {
                self::Model::from_slice(include_bytes!("../../../../res/primitives/cylinder.glb"))
            }
            Primitive::Cone => {
                self::Model::from_slice(include_bytes!("../../../../res/primitives/cone.glb"))
            }
            Primitive::Teapot => {
                self::Model::from_slice(include_bytes!("../../../../res/primitives/teapot.glb"))
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
            let translation: Vec3 = glm::make_vec3(&translation);
            let rotation = glm::make_quat(&rotation);
            let scale: Vec3 = glm::make_vec3(&scale);

            if let Some(mesh) = node.mesh() {
                let mut primitive_meshes: Vec<Mesh> = Vec::new();

                for primitive in mesh.primitives() {
                    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                    // Get vertex data from reader
                    let positions: Vec<[f32; 3]> = reader.read_positions().unwrap().collect();
                    let normals: Vec<[f32; 3]> = reader.read_normals().unwrap().collect();
                    let tex_coords: Vec<[f32; 2]> =
                        reader.read_tex_coords(0).unwrap().into_f32().collect();

                    let color = if let Some(colors) = reader.read_colors(0) {
                        let colors: Vec<[f32; 4]> = colors.into_rgba_f32().collect();
                        glm::make_vec4(&colors[0])
                    } else {
                        glm::vec4(1.0, 1.0, 1.0, 1.0)
                    };

                    let indices = if let Some(indices) = reader.read_indices() {
                        indices.into_u32().collect::<Vec<u32>>()
                    } else {
                        Vec::new()
                    };

                    // Construct vertices from the extracted data
                    let vertices: Vec<Vertex> = positions
                        .into_iter()
                        .enumerate()
                        .map(|(i, pos)| Vertex {
                            position: glm::make_vec3(&pos),
                            normal: glm::make_vec3(&normals[i]),
                            tex_uv: glm::make_vec2(&tex_coords[i]),
                            color,
                        })
                        .collect();

                    // Load textures
                    let mut textures: Vec<Rc<Texture>> = Vec::new();

                    // Load diffuse texture
                    if let Some(material) = primitive
                        .material()
                        .pbr_metallic_roughness()
                        .base_color_texture()
                    {
                        let image_index = material.texture().source().index();
                        let shared_texture = texture_cache
                            .entry(image_index)
                            .or_insert_with(|| {
                                let image = &images[image_index];
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
                                    TextureType::Diffuse,
                                    format,
                                ))
                            })
                            .clone();

                        textures.push(shared_texture);
                    }

                    // Load specular texture
                    if let Some(material) = primitive
                        .material()
                        .pbr_metallic_roughness()
                        .metallic_roughness_texture()
                    {
                        let image_index = material.texture().source().index();
                        let shared_texture = texture_cache
                            .entry(image_index)
                            .or_insert_with(|| {
                                let image = &images[image_index];
                                let format = match image.format {
                                    gltf::image::Format::R8G8B8A8 => gl::RGBA,
                                    gltf::image::Format::R8G8B8 => gl::RGB,
                                    _ => gl::RGB,
                                };
                                Rc::new(Texture::load_from_gltf(
                                    &image.pixels,
                                    image.width,
                                    image.height,
                                    TextureType::Specular,
                                    format,
                                ))
                            })
                            .clone();

                        textures.push(shared_texture);
                    }

                    // Create the mesh
                    let mesh = Mesh::new(
                        vertices,
                        indices,
                        textures,
                        mesh::MaterialProperties {
                            base_color_factor: glm::make_vec4(
                                &primitive
                                    .material()
                                    .pbr_metallic_roughness()
                                    .base_color_factor(),
                            ),
                            metallic_factor: primitive
                                .material()
                                .pbr_metallic_roughness()
                                .metallic_factor(),
                            roughness_factor: primitive
                                .material()
                                .pbr_metallic_roughness()
                                .roughness_factor(),
                            double_sided: primitive.material().double_sided(),
                            alpha_mode: match primitive.material().alpha_mode() {
                                gltf::material::AlphaMode::Opaque => mesh::AlphaMode::Opaque,
                                gltf::material::AlphaMode::Mask => mesh::AlphaMode::Mask,
                                gltf::material::AlphaMode::Blend => mesh::AlphaMode::Blend,
                            },
                            alpha_cutoff: primitive.material().alpha_cutoff().unwrap_or(0.5),
                        },
                    );
                    primitive_meshes.push(mesh);
                }

                let node = MeshNode {
                    _name: node.name().unwrap_or_default().to_string(),
                    transform: NodeTransform::new(translation, rotation, scale),
                    mesh_primitives: primitive_meshes,
                };
                nodes.push(node);
            }
        }

        Model {
            nodes,
            transform: NodeTransform::default(),
            children: NodeManager::new(),
            ready_callback: None,
            behavior_callback: None,
        }
    }

    pub fn set_material(&mut self, material: mesh::MaterialProperties) -> &mut Self {
        for node in &mut self.nodes {
            for mesh in &mut node.mesh_primitives {
                mesh.set_material(material.clone());
            }
        }
        self
    }

    /// define a callback to be called when the model is ready
    ///
    /// # Arguments
    /// - `ready_function` - the function to be called when the model is ready
    ///
    /// # Returns
    /// Self
    pub fn define_ready<F>(&mut self, ready_function: F) -> &mut Self
    where
        F: 'static + FnMut(&mut Self),
    {
        self.ready_callback = Some(Box::new(ready_function));
        self
    }

    /// define a callback to be called when the model is behaving
    ///
    /// # Arguments
    /// - `behavior_function` - the function to be called when the model is behaving
    ///
    /// # Returns
    /// Self
    pub fn define_behavior<F>(&mut self, behavior_function: F) -> &mut Self
    where
        F: 'static + FnMut(&mut Self, &mut GameContext),
    {
        self.behavior_callback = Some(Box::new(behavior_function));
        self
    }
}
