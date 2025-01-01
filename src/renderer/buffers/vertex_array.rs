//! Vertex Array Object (VAO) abstraction.
//!
//! A Vertex Array Object (VAO) is an OpenGL object that stores all of the state needed to supply vertex data to the OpenGL pipeline. It stores the format of the vertex data as well as the Buffer Objects providing the vertex data arrays.

use super::vertex_buffer::VertexBuffer;
use super::vertex_buffer_layout::{VertexBufferElement, VertexBufferLayout};

/// stores the vertex array
pub struct VertexArray {
    id: u32,
}

impl Default for VertexArray {
    fn default() -> Self {
        Self::new()
    }
}

impl VertexArray {
    /// Creates a new vertex array
    pub fn new() -> VertexArray {
        unsafe {
            let mut id = 0;
            gl::GenVertexArrays(1, &mut id);
            VertexArray { id }
        }
    }

    /// Adds a buffer to the vertex array
    ///
    /// # Arguments
    /// - `buffer` - the buffer to add
    /// - `layout` - the layout of the buffer
    pub fn add_buffer(&self, buffer: &VertexBuffer, layout: &VertexBufferLayout) {
        buffer.bind();
        self.bind();

        let mut offset = 0;
        for (i, element) in layout.elements.iter().enumerate() {
            unsafe {
                gl::EnableVertexAttribArray(i as u32);
                gl::VertexAttribPointer(
                    i as u32,
                    element.count,
                    element.type_,
                    element.normalized as u8,
                    layout.stride,
                    offset as *const std::ffi::c_void,
                );
                gl::VertexAttribDivisor(i as u32, 0);
            }

            offset += element.count * VertexBufferElement::size_of_type(element.type_);
        }

        self.unbind();
        buffer.unbind();
    }

    /// Binds the vertex array
    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }
    /// Unbinds the vertex array
    pub fn unbind(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }
}
