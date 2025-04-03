//! a single depth map used for rendering shadow maps

use crate::renderer::shader::Shader;

/// opengl depth cube map abstraction
#[derive(Clone, Debug)]
pub struct DepthCubeMap {
    framebuffer: u32,
    texture: u32,
    depth_shader: Shader,
    width: i32,
    height: i32,
}

impl DepthCubeMap {
    /// generate a new cube map
    pub fn gen_map(width: u32, height: u32, shader: Shader) -> DepthCubeMap {
        let mut framebuffer: u32 = 0;
        let mut texture: u32 = 0;

        unsafe {
            // Generate and bind the framebuffer
            gl::GenFramebuffers(1, &mut framebuffer);
            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);

            // Generate the cube map texture
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, texture);
            for i in 0..6 {
                gl::TexImage2D(
                    gl::TEXTURE_CUBE_MAP_POSITIVE_X + i as u32,
                    0,
                    gl::DEPTH_COMPONENT as i32,
                    width as i32,
                    height as i32,
                    0,
                    gl::DEPTH_COMPONENT,
                    gl::FLOAT,
                    std::ptr::null(),
                );
            }
            // Set texture parameters
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MAG_FILTER,
                gl::NEAREST as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_MIN_FILTER,
                gl::NEAREST as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_EDGE as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_EDGE as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_CUBE_MAP,
                gl::TEXTURE_WRAP_R,
                gl::CLAMP_TO_EDGE as i32,
            );

            // Attach the cube map to the framebuffer
            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, texture, 0);

            // Disable color buffers (only depth needed)
            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);

            // Check if framebuffer is complete
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                panic!("Framebuffer is not complete!");
            }

            // Unbind framebuffer
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        DepthCubeMap {
            framebuffer,
            texture,
            depth_shader: shader,
            width: width as i32,
            height: height as i32,
        }
    }
    /// binds the framebuffer
    pub fn bind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
        }
    }

    /// binds the texture attached to the framebuffer
    pub fn bind_shadow_map(&mut self, shader: &mut Shader, uniform: &str, slot: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot);
            gl::BindTexture(gl::TEXTURE_CUBE_MAP, self.texture);

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

    /// get the texture id
    pub fn get_texture(&self) -> u32 {
        self.texture
    }

    /// prepare opengl to render a depth map as well as bind the framebuffer and give a refrence to
    /// the attached shader
    pub fn prepare_shadow_map(&mut self) -> &mut Shader {
        self.bind();
        unsafe {
            gl::Enable(gl::DEPTH_TEST);

            gl::Enable(gl::BLEND);

            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            gl::Viewport(0, 0, self.width, self.height);

            self.bind();

            gl::Clear(gl::DEPTH_BUFFER_BIT);
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::FRONT);
        }
        self.depth_shader.bind();
        &mut self.depth_shader
    }

    /// reset the rendering
    pub fn finish_shadow_map(&mut self) {
        self.depth_shader.unbind();
        unsafe {
            gl::CullFace(gl::BACK);
            gl::Disable(gl::BLEND);
        }
        self.unbind();
    }

    /// renders the shadow map
    pub fn render_shadow_map(&mut self, render_function: &mut dyn FnMut(&mut Shader)) {
        self.bind();
        unsafe {
            gl::Enable(gl::DEPTH_TEST);

            gl::Enable(gl::BLEND);

            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            gl::Viewport(0, 0, self.width, self.height);

            self.bind();

            gl::Clear(gl::DEPTH_BUFFER_BIT);
            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::FRONT);
        }
        render_function(&mut self.depth_shader);
        unsafe {
            gl::CullFace(gl::BACK);
            gl::Disable(gl::BLEND);
        }
        self.unbind();
    }
}
