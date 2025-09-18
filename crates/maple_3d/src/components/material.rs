use bytemuck::Pod;
use glam as math;
use maple_renderer::{core::texture::Texture, types::lazy_buffer::LazyItemBuffer};

use std::rc::Rc;

/// how to treat alpha channel for fragment colors
#[derive(Debug, Clone, PartialEq, Copy)]
pub enum AlphaMode {
    /// mesh is opaque (cant see through it)
    Opaque,
    /// mesh is opaque to a point before being culled
    Mask,
    /// mesh opacity is same as alpha
    Blend,
}

/// Material properties for the mesh
#[derive(Debug, Clone)]
pub struct MaterialProperties {
    /// Base color factor of the material
    pub base_color_factor: math::Vec4,
    /// texture for base color
    pub base_color_texture: Option<Rc<Texture>>,

    /// Metallic factor of the material
    pub metallic_factor: f32,
    /// Roughness factor of the material
    pub roughness_factor: f32,
    /// texture for materials metallic roughness
    ///
    /// metallic on blue channel and roughness on green channel
    pub metallic_roughness_texture: Option<Rc<Texture>>,

    /// scale of objects normals
    pub normal_scale: f32,
    /// texture for normals
    pub normal_texture: Option<Rc<Texture>>,

    /// strength of ambient occlusion
    pub ambient_occlusion_strength: f32,
    /// texture for ambient occlusion
    pub occlusion_texture: Option<Rc<Texture>>,

    /// strength of an objects emission
    pub emissive_factor: math::Vec3,
    /// texture for emission
    pub emissive_texture: Option<Rc<Texture>>,

    /// Double sided property of the material
    pub double_sided: bool,
    /// Alpha mode of the material
    pub alpha_mode: AlphaMode,
    /// Alpha cutoff of the material
    pub alpha_cutoff: f32,

    uniform: LazyItemBuffer<MaterialBufferData>,
}

/// buffer data for the uniform std430
#[derive(Debug, Clone, Pod)]
#[repr(C)]
pub struct MaterialBufferData {
    pub base_color_factor: [f32; 4],
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub normal_scale: f32,
    pub ambient_occlusion_strength: f32,
    pub emissive_factor: [f32; 4],
    pub alpha_cutoff: f32,
}

impl MaterialProperties {
    // /// sets the material uniforms on the gpu
    // pub fn set_uniforms(&self, shader: &mut Shader) {
    //     shader.set_uniform("material.baseColorFactor", self.base_color_factor);
    //     if let Some(texture) = &self.base_color_texture {
    //         shader.set_uniform("material.useTexture", true);
    //         shader.set_uniform("material.baseColorTexture", 0);
    //         texture.bind(0);
    //     } else {
    //         shader.set_uniform("material.useTexture", false);
    //     }

    //     shader.set_uniform("material.metallicFactor", self.metallic_factor);
    //     shader.set_uniform("material.roughnessFactor", self.roughness_factor);
    //     if let Some(texture) = &self.metallic_roughness_texture {
    //         shader.set_uniform("material.useMetallicRoughnessTexture", true);
    //         shader.set_uniform("material.metallicRoughnessTexture", 1);
    //         texture.bind(1);
    //     } else {
    //         shader.set_uniform("material.useMetallicRoughnessTexture", false);
    //     }

    //     shader.set_uniform("material.normalScale", self.normal_scale);
    //     if let Some(texture) = &self.normal_texture {
    //         shader.set_uniform("material.useNormalTexture", true);
    //         shader.set_uniform("material.normalTexture", 2);
    //         texture.bind(2);
    //     } else {
    //         shader.set_uniform("material.useNormalTexture", false);
    //     }

    //     shader.set_uniform(
    //         "material.ambientOcclusionStrength",
    //         self.ambient_occlusion_strength,
    //     );
    //     if let Some(texture) = &self.occlusion_texture {
    //         shader.set_uniform("material.useOcclusionTexture", true);
    //         shader.set_uniform("material.occlusionTexture", 3);
    //         texture.bind(3);
    //     } else {
    //         shader.set_uniform("material.useOcclusionTexture", false);
    //     }

    //     shader.set_uniform("material.emissiveFactor", self.emissive_factor);
    //     if let Some(texture) = &self.emissive_texture {
    //         shader.set_uniform("material.useEmissiveTexture", true);
    //         shader.set_uniform("material.emissiveTexture", 4);
    //         texture.bind(4);
    //     } else {
    //         shader.set_uniform("material.useEmissiveTexture", false);
    //     }

    //     if self.alpha_mode == AlphaMode::Mask {
    //         shader.set_uniform("material.useAlphaCutoff", true);
    //         shader.set_uniform("material.alphaCutoff", self.alpha_cutoff);
    //     } else {
    //         shader.set_uniform("material.useAlphaCutoff", false);
    //     }

    //     shader.set_uniform("material.doubleSided", self.double_sided);
    // }

    // /// Creates a new MaterialProperties instance
    // ///
    // /// # Arguments
    // /// - `base_color_factor` - The base color factor of the material
    // /// - `metallic_factor` - The metallic factor of the material
    // /// - `roughness_factor` - The roughness factor of the material
    // /// - `double_sided` - The double sided property of the material
    // /// - `alpha_mode` - The alpha mode of the material
    // /// - `alpha_cutoff` - The alpha cutoff of the material
    // pub fn new(
    //     base_color_factor: math::Vec4,
    //     metallic_factor: f32,
    //     roughness_factor: f32,
    //     double_sided: bool,
    //     alpha_mode: AlphaMode,
    //     alpha_cutoff: f32,
    // ) -> MaterialProperties {
    //     MaterialProperties {
    //         base_color_factor,
    //         metallic_factor,
    //         roughness_factor,
    //         double_sided,
    //         alpha_mode,
    //         alpha_cutoff,
    //     }
    // }

    /// the rendered color if the mesh has no texture
    ///
    /// # Arguments
    /// - `base_color_factor` - The base color factor of the material
    ///
    /// # Returns
    /// Self
    pub fn with_base_color_factor(&mut self, base_color_factor: math::Vec4) -> &mut Self {
        self.base_color_factor = base_color_factor;
        self
    }

    /// the metallic factor is the shininess of the material if the object has no metallic texture
    ///
    /// # Arguments
    /// - `metallic_factor` - The metallic factor of the material
    ///
    /// # Returns
    /// Self
    pub fn with_metallic_factor(&mut self, metallic_factor: f32) -> &mut Self {
        self.metallic_factor = metallic_factor;
        self
    }

    /// the roughness factor is the shininess of the material if the object has no roughness texture
    ///
    /// # Arguments
    /// - `roughness_factor` - The roughness factor of the material
    ///
    /// # Returns
    /// Self
    pub fn with_roughness_factor(&mut self, roughness_factor: f32) -> &mut Self {
        self.roughness_factor = roughness_factor;
        self
    }

    /// if the mesh is double sided by default the renderer will render 1 side of the mesh
    ///
    /// # Arguments
    /// - `double_sided` - The double sided property of the material
    ///
    /// # Returns
    /// Self
    pub fn with_double_sided(&mut self, double_sided: bool) -> &mut Self {
        self.double_sided = double_sided;
        self
    }

    /// the alpha mode of the material (OPAQUE, MASK, BLEND)
    ///
    ///
    pub fn with_alpha_mode(&mut self, alpha_mode: AlphaMode) -> &mut Self {
        self.alpha_mode = alpha_mode;
        self
    }

    /// the alpha cutoff of the material if the node uses MASK alpha mode then the alpha cutoff is used to determine if the pixel is transparent or not
    ///
    /// # Arguments
    /// - `alpha_cutoff` - The alpha cutoff of the material
    ///
    /// # Returns
    /// Self
    pub fn with_alpha_cutoff(&mut self, alpha_cutoff: f32) -> &mut Self {
        self.alpha_cutoff = alpha_cutoff;

        self
    }
}
