//! an array of cube maps used for shadow mapping

use crate::renderer::shader::Shader;

/// an array of cube depth maps
#[derive(Clone, Debug)]
pub struct DepthCubeMapArray {
    framebuffer: u32,
    texture: u32,
    depth_shader: Shader,
    width: i32,
    height: i32,
}

impl DepthCubeMapArray {
    /// generate a depth cube map
    ///
    /// # Arguements
    /// - `width` - width of the texture
    /// - `height` - height of the texture
    /// - `layers` - size of the array because its a depth map each layer has 6 parts eg input 1
    ///     will make 6 layers for each side of the cube
    /// - `shader` - attached shader when the framebuffer is bound it will use this shader to
    ///     render with
    ///
    /// # Returns
    /// a Depth Cube Map Array
    pub fn gen_map(width: u32, height: u32, layers: u32, shader: Shader) -> DepthCubeMapArray {
        let total_layers = layers * 6; // Each point light requires 6 layers
        let mut framebuffer: u32 = 0;
        let mut texture: u32 = 0;

        unsafe {
            // Generate and bind the framebuffer
            gl::GenFramebuffers(1, &mut framebuffer);
            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);

            // Generate the cube map array texture
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP_ARRAY, texture);

            // Allocate storage for the cube map array
            gl::TexStorage3D(
                gl::TEXTURE_CUBE_MAP_ARRAY,
                1,                     // No mipmaps
                gl::DEPTH_COMPONENT24, // Depth texture format
                width as i32,
                height as i32,
                total_layers as i32, // Total layers = point lights * 6
            );

            // Set texture parameters
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP_ARRAY,
                gl::TEXTURE_MIN_FILTER,
                gl::NEAREST as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP_ARRAY,
                gl::TEXTURE_MAG_FILTER,
                gl::NEAREST as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP_ARRAY,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP_ARRAY,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP_ARRAY,
                gl::TEXTURE_WRAP_R,
                gl::CLAMP_TO_EDGE as i32,
            );

            // Attach the first layer (for the first light) to the framebuffer
            gl::FramebufferTextureLayer(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                texture,
                0, // Mipmap level 0
                0, // First layer (0 = first cube map)
            );

            // Disable color buffer
            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);

            // Check if framebuffer is complete
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                panic!("Framebuffer is not complete!");
            }

            // Unbind framebuffer
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        DepthCubeMapArray {
            framebuffer,
            texture,
            depth_shader: shader,
            width: width as i32,
            height: height as i32,
        }
    }

    /// bind the framebuffer
    pub fn bind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
        }
    }

    /// bind the texture within the framebuffer
    pub fn bind_shadow_map(&mut self, shader: &mut Shader, uniform: &str, slot: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP_ARRAY, self.texture);

            shader.bind();
            shader.set_uniform(uniform, slot as i32);
        }
    }

    /// unbind the framebuffer
    pub fn unbind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    /// returns an id to the texture
    pub fn get_texture(&self) -> u32 {
        self.texture
    }

    /// Binds the framebuffer and prepares OpenGL state for rendering shadows.
    pub fn prepare_shadow_map(&mut self, light_index: usize) -> &mut Shader {
        self.bind();

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            gl::Viewport(0, 0, self.width, self.height);
            if light_index == 0 {
                // clear from last render if first light
                gl::Clear(gl::DEPTH_BUFFER_BIT);
            }

            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::FRONT);

            self.bind();

            //Bind the correct layer for this light (6 layers per light)
            // let first_layer = light_index * 6;

            // gl::FramebufferTextureLayer(
            //     gl::FRAMEBUFFER,
            //     gl::DEPTH_ATTACHMENT,
            //     self.texture,
            //     0,
            //     first_layer as i32, // Layer index in array
            // );

            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, self.texture, 0)
        }

        self.depth_shader.bind();
        &mut self.depth_shader
    }

    /// Cleans up OpenGL state after rendering shadows.
    pub fn finish_shadow_map(&mut self) {
        self.depth_shader.unbind();
        unsafe {
            gl::CullFace(gl::BACK);
            gl::Disable(gl::BLEND);
        }
        self.unbind();
    }

    /// Renders the shadow map for a specific light source.
    pub fn render_shadow_map(
        &mut self,
        light_index: u32,
        render_function: &mut dyn FnMut(&mut Shader),
    ) {
        self.bind();

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            gl::Viewport(0, 0, self.width, self.height);
            gl::Clear(gl::DEPTH_BUFFER_BIT);

            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::FRONT);

            // Bind the correct layer for this light
            let first_layer = light_index;
            gl::FramebufferTextureLayer(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                self.texture,
                0,                  // Mipmap level
                first_layer as i32, // Layer index in array
            );
        }

        render_function(&mut self.depth_shader);

        unsafe {
            gl::CullFace(gl::BACK);
            gl::Disable(gl::BLEND);
        }

        self.unbind();
    }

    /// Renders shadows for all point lights.
    pub fn render_all_shadows(
        &mut self,
        num_lights: u32,
        mut render_function: impl FnMut(&mut Shader, u32),
    ) {
        for light_index in 0..num_lights {
            self.prepare_shadow_map(light_index as usize);
            render_function(&mut self.depth_shader, light_index);
            self.finish_shadow_map();
        }
    }
}
