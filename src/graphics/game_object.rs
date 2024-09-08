use super::buffers::{index_buffer, vertex_array, vertex_buffer, vertex_buffer_layout};
use super::renderer;
use super::shader;
use super::texture;

#[allow(dead_code)]
#[repr(C)] //this line of code took 2 months
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub tex_coords: [f32; 2],
    pub normals: [f32; 3],
}

pub struct GameObject {
    //vertices: Vec<Vertex>,
    //indices: Vec<u32>,
    va: vertex_array::VertexArray,
    //vb: vertex_buffer::VertexBuffer,
    ib: index_buffer::IndexBuffer,
    texture: texture::Texture,
    transform: glm::Mat4,
}

impl GameObject {
    ///default constructor creates quad with black color
    // pub fn default() -> GameObject {
    //     let vertices = vec![
    //         Vertex {
    //             position: [-0.5, -0.5, 0.0],
    //             color: [0.0, 0.0, 0.0, 1.0],
    //             tex_coords: [0.0, 0.0],
    //         },
    //         Vertex {
    //             position: [0.5, -0.5, 0.0],
    //             color: [0.0, 0.0, 0.0, 1.0],
    //             tex_coords: [1.0, 0.0],
    //         },
    //         Vertex {
    //             position: [0.5, 0.5, 0.0],
    //             color: [0.0, 0.0, 0.0, 1.0],
    //             tex_coords: [1.0, 1.0],
    //         },
    //         Vertex {
    //             position: [-0.5, 0.5, 0.0],
    //             color: [0.0, 0.0, 0.0, 1.0],
    //             tex_coords: [0.0, 1.0],
    //         },
    //     ];

    //     let indices = vec![0, 1, 2, 2, 3, 0]; //order in which the vertices are drawn

    //     let va = vertex_array::VertexArray::new();
    //     va.bind();

    //     let vb = vertex_buffer::VertexBuffer::new(&vertices);

    //     let mut layout = vertex_buffer_layout::VertexBufferLayout::new();
    //     layout.push::<f32>(3); //positions
    //     layout.push::<f32>(4); //color
    //     layout.push::<f32>(2); //texture coordinates
    //     va.add_buffer(&vb, &layout);

    //     let ib = index_buffer::IndexBuffer::new(&indices);

    //     va.unbind();
    //     vb.unbind();
    //     ib.unbind();

    //     println!("Default game object created");

    //     GameObject {
    //         //vertices,
    //         //indices,
    //         va,
    //         //vb,
    //         ib,
    //         texture: texture::Texture::new_empty(),
    //         transform: glm::translate(&glm::Mat4::identity(), &glm::vec3(0.0, 0.0, 0.0)),
    //     }
    // }

    ///constructor that creates a quad with a texture
    pub fn new(positions: Vec<Vertex>, indices: Vec<u32>, texture_path: &str) -> GameObject {
        let va = vertex_array::VertexArray::new();
        va.bind();

        let vb = vertex_buffer::VertexBuffer::new(&positions);

        let mut layout = vertex_buffer_layout::VertexBufferLayout::new();
        layout.push::<f32>(3); //positions (x, y, z) (location 0 in the shader)
        layout.push::<f32>(4); //color (r, g, b, a) (location 1 in the shader)
        layout.push::<f32>(2); //texture coordinates (u, v) (location 2 in the shader)
        layout.push::<f32>(3); //normals
        va.add_buffer(&vb, &layout);

        let ib = index_buffer::IndexBuffer::new(&indices);

        va.unbind();
        vb.unbind();
        ib.unbind();

        println!("Game object created");

        GameObject {
            //vertices: positions,
            //indices,
            va,
            //vb,
            ib,
            texture: texture::Texture::new(texture_path),
            transform: glm::translate(&glm::Mat4::identity(), &glm::vec3(0.0, 0.0, 0.0)),
        }
    }

    pub fn set_texture(&mut self, texture_path: &str) {
        self.texture = texture::Texture::new(texture_path);
    }

    pub fn get_texture(&self) -> &texture::Texture {
        &self.texture
    }

    pub fn get_va(&self) -> &vertex_array::VertexArray {
        &self.va
    }

    pub fn get_ib(&self) -> &index_buffer::IndexBuffer {
        &self.ib
    }

    ///function should be called every frame you want to draw the object
    pub fn draw(&self, renderer: renderer::Renderer, shader: &mut shader::Shader) {
        renderer.draw(&self.va, &self.ib, shader);
    }

    pub fn set_transform(&mut self, transform: glm::Mat4) {
        self.transform = transform;
    }

    pub fn get_transform(&self) -> glm::Mat4 {
        self.transform
    }
}
