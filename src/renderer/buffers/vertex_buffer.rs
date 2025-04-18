//! the vertex buffer stores vertex data to be used in the OpenGL pipeline

extern crate nalgebra_glm as math;
use crate::gl;

/// stores the vertex buffer
pub struct VertexBuffer {
    id: u32,
}

impl VertexBuffer {
    /// creates a new vertex buffer
    ///
    /// # Arguments
    /// - `data` - the data to store in the vertex buffer
    pub fn new<T>(data: &[T]) -> VertexBuffer {
        unsafe {
            let mut id = 0;
            gl::GenBuffers(1, &mut id);
            gl::BindBuffer(gl::ARRAY_BUFFER, id);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                std::mem::size_of_val(data) as isize,
                data.as_ptr() as *const std::ffi::c_void,
                gl::STATIC_DRAW,
            );
            VertexBuffer { id }
        }
    }

    /// binds the vertex buffer
    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.id);
        }
    }

    /// unbinds the vertex buffer
    pub fn unbind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }
}
