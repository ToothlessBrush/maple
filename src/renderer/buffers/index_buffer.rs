pub struct IndexBuffer {
    id: u32,
    count: i32,
}

impl IndexBuffer {
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

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        }
    }

    pub fn get_count(&self) -> i32 {
        self.count
    }
}
