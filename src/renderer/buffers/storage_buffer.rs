//! Storage buffers help create and write to buffers on the gpu
//!
//! Storage Buffers are like uniforms on the gpu except they can store up to 128MB of data with
//! some gpus with greater storage. they can also be written to. while a cpu can read info on the
//! buffer its highly discouraged

use std::fmt::Debug;

/// represents the Opengl Storage Buffer Object (SSBO)
pub struct StorageBuffer {
    id: u32,
    size: isize,
}

impl StorageBuffer {
    /// create a new storage buffer of a given size
    ///
    /// recommended to have a MAX_ELEMENTS * sizeof(type)
    pub fn new(size: isize) -> StorageBuffer {
        unsafe {
            let mut id = 0;
            gl::GenBuffers(1, &mut id);
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, id);
            gl::BufferData(
                gl::SHADER_STORAGE_BUFFER,
                size,
                std::ptr::null(),
                gl::DYNAMIC_DRAW,
            );
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
            StorageBuffer { id, size }
        }
    }

    /// set the data on the buffer
    ///
    /// Recommended to have type #\[repr(C)\] for struct since we pass data as pointer. Also do not
    /// use Vec3 as in glsl vec3 are vec4 aligned but are silled packed in std430 so its easier to
    /// just use vec4 with w as 0 or 1
    ///
    /// access via shader with:
    /// ```glsl
    /// layout(std430, binding = n) readonly buffer name {
    ///     int length;
    ///     T data[];
    /// };
    /// ```
    ///
    /// # Panics
    /// if size_of_val(data) > StorageBuffer.size
    pub fn set_data<T: Debug>(&self, count: i32, data: &[T]) {
        let data_size = (std::mem::size_of_val(data)) as isize;
        assert!(
            data_size + std::mem::size_of::<i32>() as isize <= self.size,
            "Data size exceeds allocated buffer size!"
        );

        // println!("{:?}", std::mem::size_of::<T>() as isize);
        unsafe {
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.id);

            // set length component
            gl::BufferSubData(
                gl::SHADER_STORAGE_BUFFER,
                0,
                std::mem::size_of::<i32>() as isize,
                &count as *const i32 as *const _,
            );
            // set data (needs to be last)
            gl::BufferSubData(
                gl::SHADER_STORAGE_BUFFER,
                16,
                data_size,
                data.as_ptr() as *const std::ffi::c_void,
            );
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
        }
    }

    /// bind the buffer to a slot
    pub fn bind(&self, binding: u32) {
        unsafe {
            gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, binding, self.id);
        }
    }

    /// unbind buffer
    pub fn unbind(&self) {
        unsafe {
            gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0);
        }
    }
}
