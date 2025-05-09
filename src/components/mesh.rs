//! Mesh module for the mesh struct and its implementation
//!
//! The `mesh` module provides a struct for managing the mesh of a model, including vertices, indices, textures, and material properties.

use nalgebra_glm as math; // Importing the nalgebra_glm crate for mathematical operations

use crate::nodes::model::Vertex;
use crate::nodes::Camera3D;
use crate::renderer::buffers::{
    index_buffer::IndexBuffer, vertex_array::VertexArray, vertex_buffer::VertexBuffer,
    vertex_buffer_layout::VertexBufferLayout,
};
use crate::renderer::{
    shader::Shader,
    texture::{Texture, TextureType},
    Renderer,
};

use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum AlphaMode {
    Opaque,
    Mask,
    Blend,
}
#[derive(Clone, Copy)]
struct MyVec(math::Vec3);

impl std::hash::Hash for MyVec {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.x.to_bits().hash(state);
        self.0.y.to_bits().hash(state);
        self.0.z.to_bits().hash(state);
    }
}

impl PartialEq for MyVec {
    fn eq(&self, other: &Self) -> bool {
        self.0.x == other.0.x && self.0.y == other.0.y && self.0.z == other.0.z
    }
}

impl Eq for MyVec {}

/// Material properties for the mesh
#[derive(Debug, Clone)]
pub struct MaterialProperties {
    /// Base color factor of the material
    pub base_color_factor: math::Vec4,
    pub base_color_texture: Option<Rc<Texture>>,

    /// Metallic factor of the material
    pub metallic_factor: f32,
    /// Roughness factor of the material
    pub roughness_factor: f32,
    // metallic on blue channel and roughness on green channel
    pub metallic_roughness_texture: Option<Rc<Texture>>,

    pub normal_scale: f32,
    pub normal_texture: Option<Rc<Texture>>,

    pub ambient_occlusion_strength: f32,
    pub occlusion_texture: Option<Rc<Texture>>,

    pub emissive_factor: math::Vec3,
    pub emissive_texture: Option<Rc<Texture>>,

    /// Double sided property of the material
    pub double_sided: bool,
    /// Alpha mode of the material
    pub alpha_mode: AlphaMode,
    /// Alpha cutoff of the material
    pub alpha_cutoff: f32,
}

impl MaterialProperties {
    pub fn set_uniforms(&self, shader: &mut Shader) {
        shader.set_uniform("material.baseColorFactor", self.base_color_factor);
        if let Some(texture) = &self.base_color_texture {
            shader.set_uniform("material.useTexture", true);
            shader.set_uniform("material.baseColorTexture", 0);
            texture.bind(0);
        } else {
            shader.set_uniform("material.useTexture", false);
        }

        shader.set_uniform("material.metallicFactor", self.metallic_factor);
        shader.set_uniform("material.roughnessFactor", self.roughness_factor);
        if let Some(texture) = &self.metallic_roughness_texture {
            shader.set_uniform("material.useMetallicRoughnessTexture", true);
            shader.set_uniform("material.metallicRoughnessTexture", 1);
            texture.bind(1);
        } else {
            shader.set_uniform("material.useMetallicRoughnessTexture", false);
        }

        shader.set_uniform("material.normalScale", self.normal_scale);
        if let Some(texture) = &self.normal_texture {
            shader.set_uniform("material.useNormalTexture", true);
            shader.set_uniform("material.normalTexture", 2);
            texture.bind(2);
        } else {
            shader.set_uniform("material.useNormalTexture", false);
        }

        shader.set_uniform(
            "material.ambientOcclusionStrength",
            self.ambient_occlusion_strength,
        );
        if let Some(texture) = &self.occlusion_texture {
            shader.set_uniform("material.useOcclusionTexture", true);
            shader.set_uniform("material.occlusionTexture", 3);
            texture.bind(3);
        } else {
            shader.set_uniform("material.useOcclusionTexture", false);
        }

        shader.set_uniform("material.emissiveFactor", self.emissive_factor);
        if let Some(texture) = &self.emissive_texture {
            shader.set_uniform("material.useEmissiveTexture", true);
            shader.set_uniform("material.emissiveTexture", 4);
            texture.bind(4);
        } else {
            shader.set_uniform("material.useEmissiveTexture", false);
        }

        if self.alpha_mode == AlphaMode::Mask {
            shader.set_uniform("material.useAlphaCutoff", true);
            shader.set_uniform("material.alphaCutoff", self.alpha_cutoff);
        } else {
            shader.set_uniform("material.useAlphaCutoff", false);
        }

        shader.set_uniform("material.doubleSided", self.double_sided);
    }

    // /// Creates a new MaterialProperties instance
    // ///
    // /// # Arguments
    // /// - `base_color_factor` - The base color factor of the material
    // /// - `metallic_factor` - The metallic factor of the material
    // /// - `roughness_factor` - The roughness factor of the material
    // /// - `double_sided` - The double sided property of the material
    // /// - `alpha_mode` - The alpha mode of the material
    // /// - `alpha_cutoff` - The alpha cutoff of the material
    // pub fn new(
    //     base_color_factor: math::Vec4,
    //     metallic_factor: f32,
    //     roughness_factor: f32,
    //     double_sided: bool,
    //     alpha_mode: AlphaMode,
    //     alpha_cutoff: f32,
    // ) -> MaterialProperties {
    //     MaterialProperties {
    //         base_color_factor,
    //         metallic_factor,
    //         roughness_factor,
    //         double_sided,
    //         alpha_mode,
    //         alpha_cutoff,
    //     }
    // }

    /// the rendered color if the mesh has no texture
    ///
    /// # Arguments
    /// - `base_color_factor` - The base color factor of the material
    ///
    /// # Returns
    /// Self
    pub fn set_base_color_factor(&mut self, base_color_factor: math::Vec4) -> &mut Self {
        self.base_color_factor = base_color_factor;
        self
    }

    /// the metallic factor is the shininess of the material if the object has no metallic texture
    ///
    /// # Arguments
    /// - `metallic_factor` - The metallic factor of the material
    ///
    /// # Returns
    /// Self
    pub fn set_metallic_factor(&mut self, metallic_factor: f32) -> &mut Self {
        self.metallic_factor = metallic_factor;
        self
    }

    /// the roughness factor is the shininess of the material if the object has no roughness texture
    ///
    /// # Arguments
    /// - `roughness_factor` - The roughness factor of the material
    ///
    /// # Returns
    /// Self
    pub fn set_roughness_factor(&mut self, roughness_factor: f32) -> &mut Self {
        self.roughness_factor = roughness_factor;
        self
    }

    /// if the mesh is double sided by default the renderer will render 1 side of the mesh
    ///
    /// # Arguments
    /// - `double_sided` - The double sided property of the material
    ///
    /// # Returns
    /// Self
    pub fn set_double_sided(&mut self, double_sided: bool) -> &mut Self {
        self.double_sided = double_sided;
        self
    }

    /// the alpha mode of the material (OPAQUE, MASK, BLEND)
    ///
    ///
    pub fn set_alpha_mode(&mut self, alpha_mode: AlphaMode) -> &mut Self {
        self.alpha_mode = alpha_mode;
        self
    }

    /// the alpha cutoff of the material if the node uses MASK alpha mode then the alpha cutoff is used to determine if the pixel is transparent or not
    ///
    /// # Arguments
    /// - `alpha_cutoff` - The alpha cutoff of the material
    ///
    /// # Returns
    /// Self
    pub fn set_alpha_cutoff(&mut self, alpha_cutoff: f32) -> &mut Self {
        self.alpha_cutoff = alpha_cutoff;
        self
    }
}

// impl Default for MaterialProperties {
//     fn default() -> Self {
//         MaterialProperties {
//             base_color_factor: math::vec4(1.0, 1.0, 1.0, 1.0), //white
//             metallic_factor: 1.0,
//             roughness_factor: 1.0,
//             double_sided: false,
//             alpha_mode: AlphaMode::Opaque,
//             alpha_cutoff: 0.5, // gltf pipeline default
//         }
//     }
// }

/// Mesh struct for managing the mesh of a model
#[derive(Clone, Debug)]
pub struct Mesh {
    pub center: math::Vec3,

    vertices: Vec<Vertex>,
    /// Indices of the mesh
    pub indices: Vec<u32>,
    /// Material properties of the mesh
    pub material_properties: MaterialProperties,
    /// Vertex array of the mesh
    vertex_array: VertexArray,
    /// Index buffer of the mesh
    index_buffer: IndexBuffer,
}

impl Mesh {
    /// Creates a new mesh
    ///
    /// Mesh Is not a Node it is a struct that holds the data for a mesh use Model to create a node with a mesh
    ///
    /// # Arguments
    /// - `vertices` - The vertices of the mesh
    /// - `indices` - The indices of the mesh
    /// - `textures` - The textures of the mesh
    /// - `material_properties` - The material properties of the mesh
    ///
    /// # Returns
    /// The new mesh
    pub fn new(
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        material_properties: MaterialProperties,
    ) -> Mesh {
        // println!("{:?}", material_properties);

        let va = VertexArray::new();

        va.bind();

        let vb = VertexBuffer::new(&vertices);

        let mut layout = VertexBufferLayout::new();
        layout.push::<f32>(3); //positions (x, y, z) (location 0 in the shader)
        layout.push::<f32>(3); //normals (location 1 in the shader)
        layout.push::<f32>(4); //color (r, g, b, a) (location 2 in the shader)
        layout.push::<f32>(2); //texture coordinates (u, v) (location 3 in the shader)
        va.add_buffer(&vb, &layout);

        let ib = IndexBuffer::new(&indices);

        va.unbind();
        vb.unbind();
        ib.unbind();

        Mesh {
            center: calculate_center(&vertices),
            vertices,
            indices,
            material_properties,
            vertex_array: va,
            index_buffer: ib,
        }
    }

    pub fn set_material(&mut self, material_properties: MaterialProperties) {
        self.material_properties = material_properties;
    }

    pub fn shade_smooth(&mut self) {
        struct SharedVertex {
            vertex: Vertex,
            division_factor: i32,
            new_index: usize,
        }

        let mut shared_vertices = HashMap::<MyVec, SharedVertex>::new();

        // collect vertices to hashmap
        for vertex in &mut self.vertices {
            let my_position = MyVec(vertex.position);

            if let Some(v) = shared_vertices.get_mut(&my_position) {
                v.vertex.position += vertex.position;
                v.division_factor += 1;
            } else {
                shared_vertices.insert(
                    MyVec(vertex.position),
                    SharedVertex {
                        vertex: vertex.clone(),
                        division_factor: 1,
                        new_index: 0, //place holder
                    },
                );
            }
        }

        let mut vertices: Vec<Vertex> = Vec::new();
        let mut position_to_index = HashMap::<MyVec, usize>::new();

        // average normals for same position vertices
        for (i, (positon, vertex)) in shared_vertices.iter_mut().enumerate() {
            vertex.vertex.normal /= vertex.division_factor as f32;
            vertex.new_index = i;
            position_to_index.insert(*positon, i);
            vertices.push(vertex.vertex);
        }

        // Update index buffer
        for index in &mut self.indices {
            let old_vertex = &self.vertices[*index as usize]; // Get the old vertex
            let my_position = MyVec(old_vertex.position);
            *index = position_to_index[&my_position] as u32; // Replace with the new index
        }

        self.vertices = vertices;

        self.reset_vao();
    }

    fn reset_vao(&mut self) {
        let va = VertexArray::new();

        va.bind();

        let vb = VertexBuffer::new(&self.vertices);

        let mut layout = VertexBufferLayout::new();
        layout.push::<f32>(3); //positions (x, y, z) (location 0 in the shader)
        layout.push::<f32>(3); //normals (location 1 in the shader)
        layout.push::<f32>(4); //color (r, g, b, a) (location 2 in the shader)
        layout.push::<f32>(2); //texture coordinates (u, v) (location 3 in the shader)
        va.add_buffer(&vb, &layout);

        let ib = IndexBuffer::new(&self.indices);

        va.unbind();
        vb.unbind();
        ib.unbind();

        self.vertex_array = va;
        self.index_buffer = ib;
    }

    /// Draw the mesh with the shader uniform and shader binding handled in Model
    ///
    /// # Arguments
    /// - `shader` - The shader to draw the mesh with
    /// - `camera` - The camera to draw the mesh with
    pub fn draw(&self, shader: &mut Shader, camera: &Camera3D) {
        //bind stuff
        shader.bind();
        self.vertex_array.bind();
        self.index_buffer.bind();

        //set the texture unifroms based on the type of texture
        // for i in 0..self.textures.len() {
        //     let tex_type = &self.textures[i].tex_type;
        //     match tex_type {
        //         TextureType::Diffuse => {
        //             shader.set_uniform("useTexture", true);
        //         }
        //         TextureType::Specular => {}
        //     }
        //     let uniform_name = tex_type.get_uniform_name();

        //     //set the unifrom for the texture in the shader
        //     //println!("setting uniform: {} to slot {}", uniform_name, i);

        //     self.textures[i].tex_unit(shader, &uniform_name, i as u32); //set the sampler2d uniform to the texture unit
        //     self.textures[i].bind(i as u32); //bind the texture to the texture unit
        // }

        let camera_pos = camera.transform.world_space().position;
        shader.set_uniform("camPos", camera_pos);

        shader.set_uniform("u_VP", camera.get_vp_matrix());

        self.material_properties.set_uniforms(shader);

        // shader.set_uniform(
        //     "baseColorFactor",
        //     self.material_properties.base_color_factor,
        // );

        // if self.material_properties.alpha_mode == AlphaMode::Mask {
        //     shader.set_uniform("useAlphaCutoff", true);
        //     shader.set_uniform("alphaCutoff", self.material_properties.alpha_cutoff);
        // }

        shader.set_uniform("u_SpecularStrength", 0.5);

        Renderer::draw(self);

        // reset stuff
        // self.textures.iter().for_each(|t| t.unbind()); //unbind the textures
        shader.set_uniform("useTexture", false); //set the useTexture uniform to false (default)
        shader.set_uniform("useAlphaCutoff", false); //set the useAlphaCutoff uniform to false (default)
    }

    /// Draw the mesh with the shadow shader uniform and shader binding handled in Model
    pub fn draw_shadow(&self, shader: &mut Shader) {
        self.vertex_array.bind();
        self.index_buffer.bind();

        // for texture in &self.textures {
        //     if texture.tex_type == TextureType::Diffuse {
        //         texture.tex_unit(shader, &texture.tex_type.get_uniform_name(), 0);
        //         texture.bind(0);
        //         shader.set_uniform("u_hasTexture", true);
        //         break;
        //     }
        // }

        if let Some(texture) = &self.material_properties.base_color_texture {
            shader.set_uniform("u_hasTexture", true);
            shader.set_uniform("u_albedoMap", 0);
            texture.bind(0);
        } else {
            shader.set_uniform("u_useTexture", false);
        }

        let base_color = self.material_properties.base_color_factor;

        shader.set_uniform("u_baseColor", base_color);

        Renderer::draw(self);

        // self.textures.iter().for_each(|t| t.unbind());
    }
}

fn calculate_center(vertices: &[Vertex]) -> math::Vec3 {
    // devide by 0 prevention
    if vertices.is_empty() {
        return math::vec3(0.0, 0.0, 0.0);
    }

    let mut sum = math::vec3(0.0, 0.0, 0.0);
    for vertex in vertices {
        sum += vertex.position;
    }
    sum / vertices.len() as f32
}
