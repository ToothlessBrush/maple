use stb_image::stb_image;
use std::ffi::CString;

pub struct Texture {
    id: u32,
    _file_path: String,
    _local_buffer: *mut u8,
    width: i32,
    height: i32,
    _bpp: i32,
}

impl Texture {
    ///Creates a new texture with null values
    pub fn new_empty() -> Texture {
        Texture {
            id: 0,
            _file_path: String::new(),
            _local_buffer: std::ptr::null_mut(),
            width: 0,
            height: 0,
            _bpp: 0,
        }
    }

    ///Creates a new texture from a file path
    pub fn new(path: &str) -> Texture {
        let mut id = 0;
        let mut width = 0;
        let mut height = 0;
        let mut _local_buffer: *mut u8 = std::ptr::null_mut();
        let mut bpp = 0;

        unsafe {
            stb_image::stbi_set_flip_vertically_on_load(1);
            let c_path = CString::new(path).expect("CString::new failed");
            _local_buffer =
                stb_image::stbi_load(c_path.as_ptr(), &mut width, &mut height, &mut bpp, 0);

            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as i32,
                width,
                height,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                _local_buffer as *const std::ffi::c_void,
            );
            gl::BindTexture(gl::TEXTURE_2D, 0);

            if _local_buffer != std::ptr::null_mut() {
                stb_image::stbi_image_free(_local_buffer as *mut std::ffi::c_void);
            } else {
                println!("Failed to load texture: {}", path);
            }
        }

        Texture {
            id: id,
            _file_path: path.to_string(),
            _local_buffer: _local_buffer,
            width: width,
            height: height,
            _bpp: 0,
        }
    }

    ///Loads a new texture from a file path
    pub fn load_new_texture(&mut self, path: &str) {
        let mut id = 0;
        let mut width = 0;
        let mut height = 0;
        let mut _local_buffer: *mut u8 = std::ptr::null_mut();
        let mut bpp = 0;

        unsafe {
            stb_image::stbi_set_flip_vertically_on_load(1);
            let c_path = CString::new(path).expect("CString::new failed");
            _local_buffer =
                stb_image::stbi_load(c_path.as_ptr(), &mut width, &mut height, &mut bpp, 0);

            gl::GenTextures(1, &mut id);
            gl::BindTexture(gl::TEXTURE_2D, id);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as i32,
                width,
                height,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                _local_buffer as *const std::ffi::c_void,
            );
            gl::BindTexture(gl::TEXTURE_2D, 0);

            if _local_buffer != std::ptr::null_mut() {
                stb_image::stbi_image_free(_local_buffer as *mut std::ffi::c_void);
            } else {
                println!("Failed to load texture: {}", path);
            }
        }

        self.id = id;
        self._file_path = path.to_string();
        self._local_buffer = _local_buffer;
        self.width = width;
        self.height = height;
        self._bpp = 0;
    }

    pub fn bind(&self, slot: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot);
            gl::BindTexture(gl::TEXTURE_2D, self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    pub fn get_width(&self) -> i32 {
        self.width
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }
}
