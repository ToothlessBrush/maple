use crate::engine::renderer::shader::Shader;

pub struct ShadowMap {
    pub framebuffer: u32,
    pub texture: u32,
    pub depth_shader: Shader,
    pub width: i32,
    pub height: i32,
}

impl ShadowMap {
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

    pub fn bind(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
        }
    }

    pub fn unbind() {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    pub fn get_texture(&self) -> u32 {
        self.texture
    }

    pub fn bind_shadow_map(&self, shader: &mut Shader, uniform: &str, slot: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot);
            gl::BindTexture(gl::TEXTURE_2D, self.texture);

            shader.bind();
            shader.set_uniform1i(uniform, slot as i32);
        }
    }

    pub fn render_shadow_map(&mut self, render_function: &mut dyn FnMut(&mut Shader)) {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Viewport(0, 0, self.width, self.height);

            self.bind();

            gl::Clear(gl::DEPTH_BUFFER_BIT);
            gl::CullFace(gl::FRONT);
        }
        render_function(&mut self.depth_shader);
        unsafe {
            gl::CullFace(gl::BACK);
        }
        Self::unbind();
    }
}
