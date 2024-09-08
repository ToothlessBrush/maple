pub struct VertexBuffer {
    id: u32,
}

impl VertexBuffer {
    pub fn new<T>(data: &[T]) -> VertexBuffer {
        unsafe {
            let mut id = 0;
            gl::GenBuffers(1, &mut id);
            gl::BindBuffer(gl::ARRAY_BUFFER, id);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (data.len() * std::mem::size_of::<T>()) as isize,
                data.as_ptr() as *const std::ffi::c_void,
                gl::STATIC_DRAW,
            );
            VertexBuffer { id }
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }
}
