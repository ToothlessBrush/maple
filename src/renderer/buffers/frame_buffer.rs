/// FrameBuffers are used to render a scene to a texture, which can then be used as a texture in the scene. This is useful for post-processing effects like bloom, depth of field, and screen-space reflections. The FrameBuffer struct is used to create and manage frame buffers in the OpenGL pipeline.

/// The FrameBuffer struct is used to create and manage frame buffers in the OpenGL pipeline
pub struct FrameBuffer {
    fbo: gl::types::GLuint,
    texture: gl::types::GLuint,
    rbo: gl::types::GLuint,
}

impl Drop for FrameBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.fbo);
            gl::DeleteTextures(1, &self.texture);
            gl::DeleteRenderbuffers(1, &self.rbo);
        }
    }
}

impl FrameBuffer {
    /// Creates a new frame buffer
    ///
    /// # Arguments
    /// - `width` - the width of the frame buffer
    /// - `height` - the height of the frame buffer
    ///
    /// # Returns
    /// The frame buffer
    pub fn new(width: i32, height: i32) -> Self {
        let mut fbo: gl::types::GLuint = 0;
        let mut texture: gl::types::GLuint = 0;
        let mut rbo: gl::types::GLuint = 0;

        unsafe {
            //generate frame buffer
            gl::GenFramebuffers(1, &mut fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, fbo);

            //generate texture for color attachment
            gl::GenTextures(1, &mut texture);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as i32,
                width,
                height,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                std::ptr::null(),
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                texture,
                0,
            );

            //create renderbuffer object for depth and stencil attachment
            gl::GenRenderbuffers(1, &mut rbo);
            gl::BindRenderbuffer(gl::RENDERBUFFER, rbo);
            gl::RenderbufferStorage(gl::RENDERBUFFER, gl::DEPTH24_STENCIL8, width, height);
            gl::FramebufferRenderbuffer(
                gl::FRAMEBUFFER,
                gl::DEPTH_STENCIL_ATTACHMENT,
                gl::RENDERBUFFER,
                rbo,
            );

            //check framebuffer
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                panic!("Framebuffer is not complete!");
            }

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        FrameBuffer { fbo, texture, rbo }
    }
}
