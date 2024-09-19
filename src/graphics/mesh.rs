use super::buffers::{
    index_buffer::IndexBuffer,
    vertex_array::VertexArray,
    vertex_buffer::{Vertex, VertexBuffer},
    vertex_buffer_layout::VertexBufferLayout,
};
use super::camera::Camera3D;
use super::shader::Shader;
use super::texture::Texture;

use std::rc::Rc; //reference counted pointer

pub struct MaterialProperties {
    pub base_color_factor: glm::Vec4,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub double_sided: bool,
    pub alpha_mode: String,
    pub alpha_cutoff: f32,
}

pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,

    textures: Vec<Rc<Texture>>, //reference to the texture which contains the type of texture and the texture itself
    base_color_factor: glm::Vec4,

    double_sided: bool,

    vertex_array: VertexArray,
    index_buffer: IndexBuffer,
}

impl Mesh {
    pub fn new(
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        textures: Vec<Rc<Texture>>,
        base_color: glm::Vec4,
        doube_sided: bool, //if the mesh is double sided
    ) -> Mesh {
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
            vertices: vertices,
            indices: indices,
            textures: textures,
            base_color_factor: base_color,
            double_sided: doube_sided,
            vertex_array: va,
            index_buffer: ib,
        }
    }

    pub fn draw(&self, shader: &mut Shader, camera: &Camera3D) {
        //bind stuff
        shader.bind();
        self.vertex_array.bind();
        self.index_buffer.bind();

        let mut num_diffuse: u32 = 0;
        let mut num_specular: u32 = 0;

        //set the texture unifroms based on the type of texture
        for i in 0..self.textures.len() {
            let tex_type = &self.textures[i].tex_type;
            let mut num: String = "0".to_string();
            if tex_type == "diffuse" {
                shader.set_uniform_bool("useTexture", true);
                num = num_diffuse.to_string();
                num_diffuse += 1;
            }
            if tex_type == "specular" {
                num = num_specular.to_string();
                num_specular += 1;
            }
            let uniform_name = format!("{}{}", tex_type, num);
            //println!("setting uniform: {}", uniform_name);

            //set the unifrom for the texture in the shader
            self.textures[i].tex_unit(shader, &uniform_name, i as u32);
            self.textures[i].bind();
        }

        if self.double_sided {
            unsafe {
                gl::Disable(gl::CULL_FACE);
            }
        }

        let camera_pos = camera.get_position();
        shader.set_uniform3f("camPos", camera_pos.x, camera_pos.y, camera_pos.z);

        shader.set_uniform_mat4f("u_VP", &camera.get_vp_matrix());

        shader.set_uniform4f(
            "baseColorFactor",
            self.base_color_factor.x,
            self.base_color_factor.y,
            self.base_color_factor.z,
            self.base_color_factor.w,
        );

        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices.len() as i32,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }

        // reset stuff
        self.textures.iter().for_each(|t| t.unbind()); //unbind the textures
        shader.set_uniform_bool("useTexture", false); //set the useTexture uniform to false (default)

        if self.double_sided {
            unsafe {
                gl::Enable(gl::CULL_FACE);
            }
        }
    }
}
