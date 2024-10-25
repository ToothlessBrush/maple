use stb_image::stb_image;
use std::ffi::CString;

use super::shader::Shader;

pub struct Texture {
    id: u32,
    pub tex_type: String,
    _file_path: String,
    _local_buffer: *mut u8,
    width: i32,
    height: i32,
    _format: u32,
    _bpp: i32,
}

impl Texture {
    ///Creates a new texture with null values
    // pub fn new_empty() -> Texture {
    //     Texture {
    //         id: 0,
    //         tex_type: "",
    //         _file_path: String::new(),
    //         _local_buffer: std::ptr::null_mut(),
    //         width: 0,
    //         height: 0,
    //         _bpp: 0,
    //         unit: 0,
    //     }
    // }

    ///Creates a new texture from a file path
    pub fn new(path: &str, tex_type: String, format: u32) -> Texture {
        let mut id = 0;
        let mut width = 0;
        let mut height = 0;
        let mut _local_buffer: *mut u8 = std::ptr::null_mut();
        let mut bpp = 0;

        if format == gl::RGB {
            bpp = 3;
        } else if format == gl::RGBA {
            bpp = 4;
        }

        unsafe {
            stb_image::stbi_set_flip_vertically_on_load(1);
            let c_path = CString::new(path).expect("CString::new failed");
            _local_buffer =
                stb_image::stbi_load(c_path.as_ptr(), &mut width, &mut height, &mut bpp, 0);

            gl::GenTextures(1, &mut id);
            //gl::ActiveTexture(gl::TEXTURE0 + slot);
            gl::BindTexture(gl::TEXTURE_2D, id);

            // configure the way the image is resized in opengl
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::NEAREST_MIPMAP_LINEAR as i32,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            // set image to repeat
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

            // let format = if bpp == 4 { gl::RGBA } else { gl::RGB };

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA8 as i32,
                width,
                height,
                0,
                format,
                gl::UNSIGNED_BYTE,
                _local_buffer as *const std::ffi::c_void,
            );

            gl::GenerateMipmap(gl::TEXTURE_2D);

            //gl::BindTexture(gl::TEXTURE_2D, 0);

            if _local_buffer != std::ptr::null_mut() {
                stb_image::stbi_image_free(_local_buffer as *mut std::ffi::c_void);
            } else {
                println!("Failed to load texture: {}", path);
            }
        }

        Texture {
            id: id,
            tex_type: tex_type,
            _file_path: path.to_string(),
            _local_buffer: _local_buffer,
            width: width,
            height: height,
            _format: format,
            _bpp: 0,
        }
    }

    //takes an array of pixels and creates a texture from it
    pub fn load_from_gltf(
        pixel: &Vec<u8>,
        width: u32,
        height: u32,
        tex_type: &str,
        format: u32,
    ) -> Texture {
        let mut bpp = 0;
        if format == gl::RGB {
            bpp = 3;
        } else if format == gl::RGBA {
            bpp = 4;
        }

        unsafe {
            let mut id: u32 = 0;
            gl::GenTextures(1, &mut id);
            //bind the texture
            gl::BindTexture(gl::TEXTURE_2D, id);

            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::NEAREST_MIPMAP_LINEAR as i32,
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            // set image to repeat
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

            let internal_format = if bpp == 4 {
                gl::RGBA8 as i32
            } else {
                gl::RGB8 as i32
            };

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                internal_format,
                width as i32,
                height as i32,
                0,
                format,
                gl::UNSIGNED_BYTE,
                pixel.as_ptr() as *const std::ffi::c_void,
            );

            //generate mipmaps
            gl::GenerateMipmap(gl::TEXTURE_2D);

            //unbind the texture
            gl::BindTexture(gl::TEXTURE_2D, 0);

            Texture {
                id: id,
                tex_type: tex_type.to_string(),
                _file_path: String::new(),
                _local_buffer: std::ptr::null_mut(),
                width: width as i32,
                height: height as i32,
                _format: format,
                _bpp: bpp,
            }
        }
    }

    pub fn tex_unit(&self, shader: &mut Shader, uniform: &str, unit: u32) {
        shader.bind();
        shader.set_uniform1i(uniform, unit as i32)
    }

    pub fn bind(&self, unit: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + unit);
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
