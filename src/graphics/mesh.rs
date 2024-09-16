use super::buffers::{
    index_buffer::IndexBuffer,
    vertex_array::VertexArray,
    vertex_buffer::{Vertex, VertexBuffer},
    vertex_buffer_layout::VertexBufferLayout,
};
use super::camera::Camera3D;
use super::shader::Shader;
use super::texture::Texture;

pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    textures: Vec<Texture>,

    vertex_array: VertexArray,
    index_buffer: IndexBuffer,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>, textures: Vec<Texture>) -> Mesh {
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

        let camera_pos = camera.get_position();
        shader.set_uniform3f("camPos", camera_pos.x, camera_pos.y, camera_pos.z);

        shader.set_uniform_mat4f("u_VP", &camera.get_vp_matrix());

        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices.len() as i32,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }
    }
}
