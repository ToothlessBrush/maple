use bytemuck::{Pod, Zeroable};
use glam::{self as math, Vec4};
use maple_renderer::core::{
    DescriptorBindingType, DescriptorSet, DescriptorSetBuilder, DescriptorSetLayout,
    DescriptorSetLayoutDescriptor, LazyBuffer, LazyBufferable, RenderContext, StageFlags,
    texture::Texture,
};
use parking_lot::RwLock;

use std::{rc::Rc, sync::OnceLock};

/// how to treat alpha channel for fragment colors
#[derive(Debug, Clone, PartialEq, Copy, Default)]
pub enum AlphaMode {
    /// mesh is opaque (cant see through it)
    #[default]
    Opaque,
    /// mesh is opaque to a point before being culled
    Mask,
    /// mesh opacity is same as alpha
    Blend,
}

/// Material properties for the mesh
pub struct MaterialProperties {
    /// Base color factor of the material
    base_color_factor: math::Vec4,
    /// texture for base color
    base_color_texture: Option<Rc<Texture>>,

    /// Metallic factor of the material
    metallic_factor: f32,
    /// Roughness factor of the material
    roughness_factor: f32,
    /// texture for materials metallic roughness
    ///
    /// metallic on blue channel and roughness on green channel
    metallic_roughness_texture: Option<Rc<Texture>>,

    /// scale of objects normals
    normal_scale: f32,
    /// texture for normals
    normal_texture: Option<Rc<Texture>>,

    /// strength of ambient occlusion
    ambient_occlusion_strength: f32,
    /// texture for ambient occlusion
    occlusion_texture: Option<Rc<Texture>>,

    /// strength of an objects emission
    emissive_factor: math::Vec3,
    /// texture for emission
    emissive_texture: Option<Rc<Texture>>,

    /// Double sided property of the material
    double_sided: bool,
    /// Alpha mode of the material
    alpha_mode: AlphaMode,
    /// Alpha cutoff of the material
    alpha_cutoff: f32,

    buffer_data: MaterialBufferData,

    uniform: LazyBuffer<MaterialBufferData>,

    descriptor: parking_lot::RwLock<Option<DescriptorSet>>,
}

/// buffer data for the uniform std430
#[derive(Debug, Clone, Copy, Pod, Default, Zeroable)]
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

impl Default for MaterialProperties {
    fn default() -> Self {
        // Default buffer data for the GPU side
        let default_data = MaterialBufferData::default();

        Self {
            base_color_factor: math::Vec4::ONE, // default white
            base_color_texture: None,

            metallic_factor: 1.0,
            roughness_factor: 1.0,
            metallic_roughness_texture: None,

            normal_scale: 1.0,
            normal_texture: None,

            ambient_occlusion_strength: 1.0,
            occlusion_texture: None,

            emissive_factor: math::Vec3::ZERO,
            emissive_texture: None,

            double_sided: false,
            alpha_mode: AlphaMode::Opaque,
            alpha_cutoff: 0.5,

            buffer_data: MaterialBufferData::default(),

            // GPU buffer data
            uniform: RenderContext::create_unifrom_buffer_lazy(&default_data),

            // no descriptor set allocated yet
            descriptor: RwLock::new(None),
        }
    }
}

/// descriptor layout of the material static so that we only allocate one
static LAYOUT: OnceLock<DescriptorSetLayout> = OnceLock::new();

impl MaterialProperties {
    /// gets the material descriptor set (lazily allocated)
    pub fn get_descriptor(&self, rcx: RenderContext) -> DescriptorSet {
        // try to read
        {
            let read_guard = self.descriptor.read();
            if let Some(d) = &*read_guard {
                rcx.sync_lazy_buffer(&self.uniform);
                return d.clone();
            }
        }

        // not allocated yet
        let mut write_guard = self.descriptor.write();
        let layout = Self::layout(&rcx);
        let buffer = rcx.get_buffer(&self.uniform);
        let set = rcx.build_descriptor_set(DescriptorSet::builder(layout).uniform(0, &buffer));

        *write_guard = Some(set.clone());
        set.clone()
    }

    pub fn layout(rcx: &RenderContext) -> &DescriptorSetLayout {
        LAYOUT.get_or_init(|| {
            rcx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
                label: Some("Material"),
                visibility: StageFlags::FRAGMENT,
                layout: &[DescriptorBindingType::UniformBuffer],
            })
        })
    }

    /// Update the internal buffer and write to the GPU
    fn update_buffer(&mut self) {
        self.buffer_data = MaterialBufferData {
            base_color_factor: self.base_color_factor.into(),
            metallic_factor: self.metallic_factor,
            roughness_factor: self.roughness_factor,
            normal_scale: self.normal_scale,
            ambient_occlusion_strength: self.ambient_occlusion_strength,
            emissive_factor: [
                self.emissive_factor.x,
                self.emissive_factor.y,
                self.emissive_factor.z,
                0.0,
            ],
            alpha_cutoff: self.alpha_cutoff,
        };
        self.uniform.write(&self.buffer_data);
    }

    /// Base color factor (vec4)
    pub fn with_base_color_factor(mut self, base_color_factor: math::Vec4) -> Self {
        self.base_color_factor = base_color_factor;
        self.update_buffer();
        self
    }

    pub fn base_color_factor(&self) -> math::Vec4 {
        self.base_color_factor
    }

    /// Base color texture
    pub fn with_base_color_texture(mut self, texture: Option<Rc<Texture>>) -> Self {
        self.base_color_texture = texture;
        self
    }

    pub fn base_color_texture(&self) -> Option<&Rc<Texture>> {
        self.base_color_texture.as_ref()
    }

    /// Metallic factor
    pub fn with_metallic_factor(mut self, metallic_factor: f32) -> Self {
        self.metallic_factor = metallic_factor;
        self.update_buffer();
        self
    }

    pub fn metallic_factor(&self) -> f32 {
        self.metallic_factor
    }

    /// Roughness factor
    pub fn with_roughness_factor(mut self, roughness_factor: f32) -> Self {
        self.roughness_factor = roughness_factor;
        self.update_buffer();
        self
    }

    pub fn roughness_factor(&self) -> f32 {
        self.roughness_factor
    }

    /// Metallic/Roughness texture
    pub fn with_metallic_roughness_texture(mut self, texture: Option<Rc<Texture>>) -> Self {
        self.metallic_roughness_texture = texture;
        self
    }

    pub fn metallic_roughness_texture(&self) -> Option<&Rc<Texture>> {
        self.metallic_roughness_texture.as_ref()
    }

    /// Normal scale
    pub fn with_normal_scale(mut self, normal_scale: f32) -> Self {
        self.normal_scale = normal_scale;
        self.update_buffer();
        self
    }

    pub fn normal_scale(&self) -> f32 {
        self.normal_scale
    }

    /// Normal texture
    pub fn with_normal_texture(mut self, texture: Option<Rc<Texture>>) -> Self {
        self.normal_texture = texture;
        self
    }

    pub fn normal_texture(&self) -> Option<&Rc<Texture>> {
        self.normal_texture.as_ref()
    }

    /// Ambient occlusion strength
    pub fn with_ambient_occlusion_strength(mut self, strength: f32) -> Self {
        self.ambient_occlusion_strength = strength;
        self.update_buffer();
        self
    }

    pub fn ambient_occlusion_strength(&self) -> f32 {
        self.ambient_occlusion_strength
    }

    /// Occlusion texture
    pub fn with_occlusion_texture(mut self, texture: Option<Rc<Texture>>) -> Self {
        self.occlusion_texture = texture;
        self
    }

    pub fn occlusion_texture(&self) -> Option<&Rc<Texture>> {
        self.occlusion_texture.as_ref()
    }

    /// Emissive factor
    pub fn with_emissive_factor(mut self, emissive_factor: math::Vec3) -> Self {
        self.emissive_factor = emissive_factor;
        self.update_buffer();
        self
    }

    pub fn emissive_factor(&self) -> math::Vec3 {
        self.emissive_factor
    }

    /// Emissive texture
    pub fn with_emissive_texture(mut self, texture: Option<Rc<Texture>>) -> Self {
        self.emissive_texture = texture;
        self
    }

    pub fn emissive_texture(&self) -> Option<&Rc<Texture>> {
        self.emissive_texture.as_ref()
    }

    /// Double sided
    pub fn with_double_sided(mut self, double_sided: bool) -> Self {
        self.double_sided = double_sided;
        self
    }

    pub fn double_sided(&self) -> bool {
        self.double_sided
    }

    /// Alpha mode
    pub fn with_alpha_mode(mut self, alpha_mode: AlphaMode) -> Self {
        self.alpha_mode = alpha_mode;
        self
    }

    pub fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    /// Alpha cutoff
    pub fn with_alpha_cutoff(mut self, alpha_cutoff: f32) -> Self {
        self.alpha_cutoff = alpha_cutoff;
        self.update_buffer();
        self
    }

    pub fn alpha_cutoff(&self) -> f32 {
        self.alpha_cutoff
    }
}
