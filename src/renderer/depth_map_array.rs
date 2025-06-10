//! array of 2d depth maps used for shadow mapping

use std::collections::HashSet;

use crate::gl;
use crate::renderer::shader::Shader;

/// The ShadowMap struct is used to create and manage shadow maps
#[derive(Clone, Debug)]
pub struct DepthMapArray {
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

    commited_layers: std::collections::HashSet<u32>,
}

/// renderdoc doesnt support sparse texutures so heres a simple work around
const RENDERDOC_MODE: bool = true;

impl DepthMapArray {
    /// Generates a new shadow map
    ///
    /// # Arguments
    /// - `width` - the width of the shadow map
    /// - `height` - the height of the shadow map
    /// - `depth_shader` - the depth shader
    ///
    /// # Returns
    /// The shadow map
    pub fn gen_map(width: i32, height: i32, layers: usize, depth_shader: Shader) -> DepthMapArray {
        let mut framebuffer: u32 = 0;
        let mut shadow_map: u32 = 0;

        unsafe {
            // Generate framebuffer
            gl::GenFramebuffers(1, &mut framebuffer);

            // Generate texture
            gl::GenTextures(1, &mut shadow_map);
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, shadow_map);

            if !RENDERDOC_MODE {
                gl::TexParameteri(
                    gl::TEXTURE_2D_ARRAY,
                    gl::TEXTURE_SPARSE_ARB,
                    gl::TRUE.into(),
                );
            }

            let mut max_sparse_texture_size = std::mem::MaybeUninit::<i32>::uninit();
            gl::GetIntegerv(
                gl::MAX_SPARSE_TEXTURE_SIZE_ARB,
                max_sparse_texture_size.as_mut_ptr(),
            );
            let max_sparse_texture_size = max_sparse_texture_size.assume_init();

            let mut max_sparse_array_texture_layers = std::mem::MaybeUninit::<i32>::uninit();
            gl::GetIntegerv(
                gl::MAX_SPARSE_ARRAY_TEXTURE_LAYERS_ARB,
                max_sparse_array_texture_layers.as_mut_ptr(),
            );

            gl::TexParameteri(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_MIN_FILTER,
                gl::NEAREST as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_MAG_FILTER,
                gl::NEAREST as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_BORDER as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_BORDER as i32,
            );

            let clamp_color = [1.0, 1.0, 1.0, 1.0];
            gl::TexParameterfv(
                gl::TEXTURE_2D_ARRAY,
                gl::TEXTURE_BORDER_COLOR,
                clamp_color.as_ptr(),
            );

            // generate the texture
            gl::TexStorage3D(
                gl::TEXTURE_2D_ARRAY,
                1,
                gl::DEPTH_COMPONENT32F,
                width,
                height,
                std::cmp::min(max_sparse_texture_size, layers as i32),
            );

            //attach generated texture to framebuffer
            gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer);
            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::DEPTH_ATTACHMENT, shadow_map, 0);
            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);

            // Check framebuffer
            if gl::CheckFramebufferStatus(gl::FRAMEBUFFER) != gl::FRAMEBUFFER_COMPLETE {
                panic!("Framebuffer is not complete!");
            }

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }

        DepthMapArray {
            framebuffer,
            texture: shadow_map,
            depth_shader,
            width,
            height,
            commited_layers: HashSet::new(),
        }
    }

    /// commit a layer to memory
    pub fn commit_layer(&mut self, layer: u32, depth: i32) {
        if !self.commited_layers.insert(layer) {
            return;
        };

        if !RENDERDOC_MODE {
            unsafe {
                gl::TexturePageCommitmentEXT(
                    self.texture,
                    0,
                    0,
                    0,
                    layer as i32,
                    self.width,
                    self.height,
                    depth,
                    gl::TRUE,
                );
            }
        }
    }

    /// remove a layer from memory
    pub fn decommit_layer(&mut self, layer: u32, depth: i32) {
        if !self.commited_layers.remove(&layer) {
            return;
        }

        self.bind_texture();
        unsafe {
            gl::TexPageCommitmentARB(
                gl::TEXTURE_2D_ARRAY,
                0,
                0,
                0,
                layer as i32,
                self.width,
                self.height,
                depth,
                gl::FALSE,
            );
        }
    }

    /// Binds the shadow map
    pub fn bind_framebuffer(&self) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
        }
    }

    /// binds the texture
    pub fn bind_texture(&self) {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.texture);
        }
    }

    /// Binds a specific layer of the shadow map array for rendering
    pub fn bind_layer(&self, layer: i32) {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.framebuffer);
            gl::FramebufferTextureLayer(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                self.texture, // The 2D array texture
                0,            // Mipmap level
                layer,        // Layer to bind
            );
        }
    }

    /// Unbinds the shadow map
    pub fn unbind_framebuffer() {
        unsafe {
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
        }
    }

    /// unbinds the texture
    pub fn unbind_texture() {
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, 0);
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
            gl::BindTexture(gl::TEXTURE_2D_ARRAY, self.texture);

            shader.bind();
            shader.set_uniform(uniform, slot as i32);
        }
    }

    /// configures the api for rendering a depth map and passes ownership of the shader for
    /// rendering
    pub fn prepare_shadow_map(&mut self) -> Shader {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            gl::Viewport(0, 0, self.width, self.height);

            self.bind_framebuffer();

            gl::Clear(gl::DEPTH_BUFFER_BIT);

            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::FRONT);
        }

        self.depth_shader.bind();
        std::mem::take(&mut self.depth_shader)
    }

    /// resets the api
    ///
    /// # Arguements
    /// - `depth_shader` - reinput the shader from the prep
    pub fn finish_shadow_map(&mut self, depth_shader: Shader) {
        self.depth_shader = depth_shader;

        self.depth_shader.unbind();

        unsafe {
            gl::CullFace(gl::BACK);
            gl::Disable(gl::BLEND);
        }

        Self::unbind_framebuffer();
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

            self.bind_framebuffer();

            gl::Clear(gl::DEPTH_BUFFER_BIT);

            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::FRONT);
        }
        render_function(&mut self.depth_shader);
        unsafe {
            gl::CullFace(gl::BACK);
            gl::Disable(gl::BLEND);
        }
        Self::unbind_framebuffer();
    }
}
