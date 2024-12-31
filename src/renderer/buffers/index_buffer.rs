/// the index buffer is used to store the indices of the vertices

/// stores the index buffer
pub struct IndexBuffer {
    id: u32,
    count: i32,
}

impl IndexBuffer {
    /// Creates a new index buffer
    ///
    /// # Arguments
    /// - `data` - the data to store in the index buffer
    pub fn new(data: &[u32]) -> IndexBuffer {
        unsafe {
            let mut id = 0;
            gl::GenBuffers(1, &mut id);
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, id);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                std::mem::size_of_val(data) as isize,
                data.as_ptr() as *const std::ffi::c_void,
                gl::STATIC_DRAW,
            );
            IndexBuffer {
                id,
                count: data.len() as i32,
            }
        }
    }

    /// Binds the index buffer
    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.id);
        }
    }

    /// Unbinds the index buffer
    pub fn unbind(&self) {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
    }

    /// Gets the count of the index buffer
    pub fn get_count(&self) -> i32 {
        self.count
    }
}
