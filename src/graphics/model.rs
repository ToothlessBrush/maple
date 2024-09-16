extern crate gltf;
use glm::{Mat4, Vec3, Vec4};
use gltf::{image::Source, scene::Transform};
use std::{path::Path, primitive};

use super::{
    buffers::{self, vertex_buffer::Vertex},
    camera::Camera3D,
    mesh::Mesh,
    shader::Shader,
    texture::Texture,
};

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
    name: String,
    transform: NodeTransform,
    transform_matrix: Mat4,
    mesh_primitives: Vec<Mesh>,
}

pub struct Model {
    nodes: Vec<Node>,
}

impl Model {
    pub fn new(file: &str) -> Model {
        let gltf = gltf::import(Path::new(file)).expect("failed to open GLTF file");
        let (doc, buffers, images) = gltf;

        let mut nodes: Vec<Node> = Vec::new();

        for node in doc.nodes() {
            println!("loading Node: {:?}", node.name());
            //get node transformation data
            let mut matrix = Mat4::identity();
            let (translation, rotation, scale) = node.transform().decomposed();
            let translation: Vec3 = glm::make_vec3(&translation);
            let rotation: Vec4 = glm::make_vec4(&rotation);
            let scale: Vec3 = glm::make_vec3(&scale);

            println!("translation: {:?}", translation);
            println!("rotation: {:?}", rotation);
            println!("scale: {:?}", scale);

            let quat_rotation = glm::quat(rotation.x, rotation.y, rotation.z, rotation.w);

            //get matrix from translation, rotation, and scale
            matrix += glm::translate(&Mat4::identity(), &translation);
            matrix += glm::quat_to_mat4(&quat_rotation);
            matrix += glm::scale(&Mat4::identity(), &scale);

            if let Some(mesh) = node.mesh() {
                let mut primitive_meshes: Vec<Mesh> = Vec::new();
                for primitive in mesh.primitives() {
                    println!("loading Primitive: {:?}", primitive.index());
                    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                    //get vertex data from reader
                    let positions: Vec<[f32; 3]> = reader.read_positions().unwrap().collect();
                    let normals: Vec<[f32; 3]> = reader.read_normals().unwrap().collect();
                    let tex_coords: Vec<[f32; 2]> =
                        reader.read_tex_coords(0).unwrap().into_f32().collect();
                    let color: Vec4 = glm::vec4(1.0, 1.0, 1.0, 1.0);

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
                            texUV: glm::make_vec2(&tex_coords[i]),
                            color,
                        })
                        .collect();

                    //load textures
                    let mut textures: Vec<Texture> = Vec::new();

                    //load diffuse texture
                    if let Some(material) = primitive
                        .material()
                        .pbr_metallic_roughness()
                        .base_color_texture()
                    {
                        println!("loading diffuse texture");
                        let image_index = material.texture().source().index();
                        let image = &images[image_index];
                        let format = if image.format == gltf::image::Format::R8G8B8A8 {
                            gl::RGBA
                        } else {
                            gl::RGB
                        };
                        let texture = Texture::load_from_gltf(
                            &image.pixels,
                            image.width,
                            image.height,
                            "diffuse",
                            format,
                        );
                        textures.push(texture);
                    }

                    //load specular texture
                    if let Some(material) = primitive
                        .material()
                        .pbr_metallic_roughness()
                        .metallic_roughness_texture()
                    {
                        println!("loading specular texture");
                        let image_index = material.texture().source().index();
                        let image = &images[image_index];
                        let format = if image.format == gltf::image::Format::R8G8B8A8 {
                            gl::RGBA
                        } else {
                            gl::RGB
                        };
                        let texture = Texture::load_from_gltf(
                            &image.pixels,
                            image.width,
                            image.height,
                            "specular",
                            format,
                        );
                        textures.push(texture);
                    }

                    //create the mesh
                    let mesh = Mesh::new(vertices, indices, textures);
                    primitive_meshes.push(mesh);
                }

                let node = Node {
                    name: node.name().unwrap_or_default().to_string(),
                    transform: NodeTransform {
                        translation,
                        rotation: quat_rotation,
                        scale,
                    },
                    transform_matrix: matrix,
                    mesh_primitives: primitive_meshes,
                };
                nodes.push(node);
            }
        }

        println!("successfully loaded model: {}", file);
        Model { nodes: nodes }
    }

    pub fn draw(&self, shader: &mut Shader, camera: &Camera3D) {
        for node in &self.nodes {
            shader.bind();
            shader.set_uniform_mat4f("u_Model", &node.transform_matrix);

            for mesh in &node.mesh_primitives {
                mesh.draw(shader, camera);
            }
        }
    }

    pub fn translate(&mut self, translation: Vec3) {
        for node in &mut self.nodes {
            node.transform.translation += translation;
            node.transform_matrix = glm::translate(&node.transform_matrix, &translation);
        }
    }

    pub fn rotate(&mut self, axis: Vec3, degrees: f32) {
        let radians = glm::radians(&glm::vec1(degrees)).x;

        let rotation_quat = glm::quat_angle_axis(radians, &axis);

        for node in &mut self.nodes {
            node.transform.rotation = rotation_quat * node.transform.rotation;
            node.transform_matrix = glm::quat_to_mat4(&rotation_quat) * node.transform_matrix;
        }
    }

    pub fn scale(&mut self, scale: Vec3) {
        for node in &mut self.nodes {
            node.transform.scale += scale;
            node.transform_matrix = glm::scale(&node.transform_matrix, &scale);
        }
    }
}
