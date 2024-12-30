use super::vertex_buffer::VertexBuffer;
use super::vertex_buffer_layout::{VertexBufferElement, VertexBufferLayout};

pub struct VertexArray {
    id: u32,
}

impl Default for VertexArray {
    fn default() -> Self {
        Self::new()
    }
}

impl VertexArray {
    pub fn new() -> VertexArray {
        unsafe {
            let mut id = 0;
            gl::GenVertexArrays(1, &mut id);
            VertexArray { id }
        }
    }

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

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }
}
