use nalgebra_glm as glm;

extern crate gltf;
use glm::{Mat4, Vec3, Vec4};
use std::io::Write;
use std::{collections::BTreeMap, collections::HashMap, path::Path, rc::Rc};

use colored::*;

use std::thread;
use std::time::Duration;

use crate::engine::game_context::GameContext;

use crate::engine::renderer::{shader::Shader, texture::Texture};

use super::{camera::Camera3D, mesh, mesh::Mesh};

#[derive(Debug)]
#[repr(C)]
pub struct Vertex {
    pub position: glm::Vec3,
    pub normal: glm::Vec3,
    pub color: glm::Vec4,
    pub tex_uv: glm::Vec2,
}

struct NodeTransform {
    translation: Vec3,
    rotation: glm::Quat,
    scale: Vec3,
}

// struct MeshPrimitive {
//     vertices: Vec<Vertex>,
//     indices: Vec<u32>,
//     textures: Vec<Texture>,
// }

struct Node {
    _name: String,
    transform: NodeTransform,
    transform_matrix: Mat4,
    mesh_primitives: Vec<Mesh>,
}

pub struct Model {
    nodes: Vec<Node>,
    ready_callback: Option<Box<dyn FnMut(&mut Model)>>,
    behavior_callback: Option<Box<dyn FnMut(&mut Model, &mut GameContext)>>,
}

impl Model {
    pub fn new(file: &str) -> Model {
        let model_loaded = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let model_loaded_clone = model_loaded.clone();
        thread::spawn(move || {
            let animation = ["\\", "|", "/", "-"];
            let mut i = 0;
            while !model_loaded.load(std::sync::atomic::Ordering::SeqCst) {
                print!("{}", format!("\rloading model: {}", animation[i]).cyan());
                std::io::stdout().flush().unwrap();
                i = (i + 1) % 4;

                thread::sleep(Duration::from_millis(50));
            }
            print!("\rloading model: done\n");
            std::io::stdout().flush().unwrap();
        });

        let gltf = gltf::import(Path::new(file)).expect("failed to open GLTF file");
        let (doc, buffers, images) = gltf;

        //end thread here
        model_loaded_clone.store(true, std::sync::atomic::Ordering::SeqCst);

        let mut nodes: Vec<Node> = Vec::new();

        let mut texture_cache: HashMap<usize, Rc<Texture>> = HashMap::new(); //cache with key as image index and value as a smart pointer to the texture

        for node in doc.nodes() {
            println!("loading Node: {:?}", node.name().unwrap());
            //get node transformation data
            let (translation, rotation, scale) = node.transform().decomposed();
            let translation: Vec3 = glm::make_vec3(&translation);
            let rotation: Vec4 = glm::make_vec4(&rotation);
            let scale: Vec3 = glm::make_vec3(&scale);

            let quat_rotation = glm::quat(rotation.x, rotation.y, rotation.z, rotation.w);

            let translation_matrix = glm::translate(&Mat4::identity(), &translation);
            let rotation_matrix = glm::quat_to_mat4(&quat_rotation);
            let scale_matrix = glm::scale(&Mat4::identity(), &scale);

            //get matrix from translation, rotation, and scale
            let matrix: glm::Mat4 = scale_matrix * rotation_matrix * translation_matrix; //scale the rotatation and translation

            if let Some(mesh) = node.mesh() {
                let mut primitive_meshes: Vec<Mesh> = Vec::new();
                for primitive in mesh.primitives() {
                    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                    //get vertex data from reader
                    let positions: Vec<[f32; 3]> = reader.read_positions().unwrap().collect();
                    let normals: Vec<[f32; 3]> = reader.read_normals().unwrap().collect();
                    let tex_coords: Vec<[f32; 2]> =
                        reader.read_tex_coords(0).unwrap().into_f32().collect();
                    //read color data if it exists otherwise set color to white
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

                    //construct vertices from the extracted data
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

                    //load textures
                    let mut textures: Vec<Rc<Texture>> = Vec::new();

                    //load diffuse texture
                    if let Some(material) = primitive
                        .material()
                        .pbr_metallic_roughness()
                        .base_color_texture()
                    {
                        let image_index = material.texture().source().index();
                        let shared_texture = texture_cache //check if the texture is already loaded if so then use the cached texture to avoid loading the same texture multiple times
                            .entry(image_index)
                            .or_insert_with(|| {
                                let image = &images[image_index];
                                let format = if image.format == gltf::image::Format::R8G8B8A8 {
                                    gl::RGBA
                                } else if image.format == gltf::image::Format::R8G8B8 {
                                    gl::RGB
                                } else if image.format == gltf::image::Format::R8 {
                                    gl::RED
                                } else {
                                    panic!("unsupported image format not rgba, rgb, or r");
                                };
                                Rc::new(Texture::load_from_gltf(
                                    &image.pixels,
                                    image.width,
                                    image.height,
                                    "diffuse",
                                    format,
                                ))
                            })
                            .clone();

                        textures.push(shared_texture);
                    };

                    //load specular texture (we load the metallic roughness texture as the specular texture since metallic roughtness is the closest thing to specular in gltf)
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
                                let format = if image.format == gltf::image::Format::R8G8B8A8 {
                                    //rgba format
                                    gl::RGBA
                                } else if image.format == gltf::image::Format::R8G8B8 {
                                    //rgb format
                                    gl::RGB
                                } else {
                                    gl::RGB
                                };
                                Rc::new(Texture::load_from_gltf(
                                    &image.pixels,
                                    image.width,
                                    image.height,
                                    "specular",
                                    format,
                                ))
                            })
                            .clone();

                        textures.push(shared_texture);
                    }

                    //create the mesh
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
                                gltf::material::AlphaMode::Opaque => "OPAQUE".to_string(),
                                gltf::material::AlphaMode::Mask => "MASK".to_string(),
                                gltf::material::AlphaMode::Blend => "BLEND".to_string(),
                            },
                            alpha_cutoff: primitive.material().alpha_cutoff().unwrap_or(0.5),
                        },
                    );
                    primitive_meshes.push(mesh);
                }

                let node = Node {
                    _name: node.name().unwrap_or_default().to_string(),
                    transform: NodeTransform {
                        translation,
                        rotation: quat_rotation,
                        scale,
                    },
                    transform_matrix: matrix,
                    mesh_primitives: primitive_meshes,
                };
                nodes.push(node);

                println!("successfully loaded model: {}", file);
            }
        }

        println!("successfully loaded model: {}", file);
        Model {
            nodes,
            ready_callback: None,
            behavior_callback: None,
        }
    }

    pub fn draw(&mut self, shader: &mut Shader, camera: &Camera3D) {
        let mut sorted_nodes = BTreeMap::new();

        for node in &self.nodes {
            let position = node.transform.translation;
            let distance = glm::length(&(camera.get_position() - position)) as i32;
            sorted_nodes.insert(distance, node);
        }

        for node in &self.nodes {
            shader.bind();
            shader.set_uniform_mat4f("u_Model", &node.transform_matrix);
            //println!("drawing node: {}", node.transform_matrix);

            for mesh in &node.mesh_primitives {
                mesh.draw(shader, camera);
            }
        }
    }

    pub fn draw_shadow(&mut self, depth_shader: &mut Shader, light_space_matrix: &Mat4) {
        for node in &self.nodes {
            depth_shader.bind();
            depth_shader.set_uniform_mat4f("u_lightSpaceMatrix", light_space_matrix);
            depth_shader.set_uniform_mat4f("u_Model", &node.transform_matrix);

            for mesh in &node.mesh_primitives {
                mesh.draw_shadow();
            }
        }
    }

    pub fn translate(&mut self, translation: Vec3) -> &mut Model {
        for node in &mut self.nodes {
            node.transform.translation += translation;
            node.transform_matrix = glm::translate(&node.transform_matrix, &translation);
        }
        self
    }

    pub fn rotate(&mut self, axis: Vec3, degrees: f32) -> &mut Model {
        let radians = glm::radians(&glm::vec1(degrees)).x;

        let rotation_quat = glm::quat_angle_axis(radians, &axis);

        for node in &mut self.nodes {
            node.transform.rotation = rotation_quat * node.transform.rotation;
            node.transform_matrix = glm::quat_to_mat4(&rotation_quat) * node.transform_matrix;
        }

        self
    }

    pub fn rotate_euler_xyz(&mut self, degrees: Vec3) -> &mut Model {
        let radians = glm::radians(&degrees);

        let rotation_quat_x = glm::quat_angle_axis(radians.x, &glm::vec3(1.0, 0.0, 0.0));
        let rotation_quat_y = glm::quat_angle_axis(radians.y, &glm::vec3(0.0, 1.0, 0.0));
        let rotation_quat_z = glm::quat_angle_axis(radians.z, &glm::vec3(0.0, 0.0, 1.0));

        let rotation_quat = rotation_quat_x * rotation_quat_y * rotation_quat_z; //multiply in a xyz order

        for node in &mut self.nodes {
            //update the rotation quaternion and matrix
            node.transform.rotation = rotation_quat * node.transform.rotation;
            let rotation_matrix = glm::quat_to_mat4(&rotation_quat);
            node.transform_matrix = rotation_matrix * node.transform_matrix;
        }

        self
    }

    pub fn scale(&mut self, scale: Vec3) -> &mut Model {
        for node in &mut self.nodes {
            node.transform.scale += scale;
            node.transform_matrix = glm::scale(&node.transform_matrix, &scale);
        }
        self
    }

    pub fn define_ready<F>(&mut self, ready_function: F) -> &mut Model
    where
        F: FnMut(&mut Model) + 'static,
    {
        self.ready_callback = Some(Box::new(ready_function));
        self
    }

    pub fn define_behavior<F>(&mut self, behavior_function: F) -> &mut Model
    where
        F: FnMut(&mut Model, &mut GameContext) + 'static,
    {
        self.behavior_callback = Some(Box::new(behavior_function));
        self
    }

    //if the model has a ready function then call it
    pub fn ready(&mut self) {
        if let Some(mut callback) = self.ready_callback.take() {
            callback(self);
            self.ready_callback = Some(callback);
        }
    }

    //if the model has a behavior function then call it
    pub fn behavior(&mut self, context: &mut GameContext) {
        if let Some(mut callback) = self.behavior_callback.take() {
            callback(self, context);
            self.behavior_callback = Some(callback);
        }
    }
}
