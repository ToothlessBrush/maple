//! shadow maps store depth information from the light's perspective to render shadows at the draw stage
use crate::renderer::shader::Shader;

use crate::gl;

/// The ShadowMap struct is used to create and manage shadow maps
#[derive(Clone, Debug)]
pub struct ShadowMap {
    /// The framebuffer object
    pub framebuffer: u32,
    /// The shadow map texture
    pub texture: u32,
    /// The depth shader
    pub depth_shader: Shader,
    /// The width of the shadow map
    pub width: i32,
    /// The height of the shadow map
    pub height: i32,
}

impl ShadowMap {
    /// Generates a new shadow map
    ///
    /// # Arguments
    /// - `width` - the width of the shadow map
    /// - `height` - the height of the shadow map
    /// - `depth_shader` - the depth shader
    ///
    /// # Returns
    /// The shadow map
    pub fn gen_map(width: i32, height: i32, depth_shader: Shader) -> ShadowMap {
        let mut framebuffer: u32 = 0;
        let mut shadow_map: u32 = 0;

        unsafe {
            // Generate framebuffer
            gl::GenFramebuffers(1, &mut framebuffer);

            // Generate texture
            gl::GenTextures(1, &mut shadow_map);
            gl::BindTexture(gl::TEXTURE_2D, shadow_map);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::DEPTH_COMPONENT as i32,
                width,
                height,
                0,
                gl::DEPTH_COMPONENT,
                gl::FLOAT,
                std::ptr::null(),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_BORDER as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_BORDER as i32,
            );

            let clamp_color = [1.0, 1.0, 1.0, 1.0];
            gl::TexParameterfv(
                gl::TEXTURE_2D,
                gl::TEXTURE_BORDER_COLOR,
                clamp_color.as_ptr(),
            );

            //attach generated texture to framebuffer
            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::TEXTURE_2D,
                shadow_map,
                0,
            );
            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);

            // Check framebuffer
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                panic!("Framebuffer is not complete!");
            }

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        ShadowMap {
            framebuffer,
            texture: shadow_map,
            depth_shader,
            width,
            height,
        }
    }

    /// Binds the shadow map
    pub fn bind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
        }
    }
    /// Unbinds the shadow map
    pub fn unbind() {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }
    /// Gets the shadow map texture
    ///
    /// # Returns
    /// The shadow map texture
    pub fn get_texture(&self) -> u32 {
        self.texture
    }

    /// Binds the shadow map to a shader
    ///
    /// # Arguments
    /// - `shader` - the shader to bind the shadow map to
    /// - `uniform` - the uniform to bind the shadow map to
    /// - `slot` - the texture slot to bind the shadow map to
    pub fn bind_shadow_map(&self, shader: &mut Shader, uniform: &str, slot: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);

            shader.bind();
            shader.set_uniform(uniform, slot as i32);
        }
    }

    /// configures the graphics api for depth mapping
    pub fn prepare_shadow_map(&mut self) -> &mut Shader {
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

    /// resets the api from shadow mapping
    pub fn finish_shadow_map(&mut self) {
        self.depth_shader.unbind();

        unsafe {
            gl::CullFace(gl::BACK);
            gl::Disable(gl::BLEND);
        }

        Self::unbind();
    }

    /// Renders the shadow map
    ///
    /// # Arguments
    /// - `render_function` - the render function to render the shadow map
    pub fn render_shadow_map(&mut self, render_function: &mut dyn FnMut(&mut Shader)) {
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
        Self::unbind();
    }
}
