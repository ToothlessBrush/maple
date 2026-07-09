use bytemuck::{Pod, Zeroable};
use glam::{self as math, Vec2};
use maple_engine::{
    asset::{AssetHandle, AssetLibrary, AssetRef, AssetState, IntoAsset},
    utils::Color,
};
use maple_renderer::core::{
    Buffer, DescriptorBindingType, DescriptorSet, DescriptorSetLayout,
    DescriptorSetLayoutDescriptor, RenderContext, StageFlags, texture::Texture,
};

use std::sync::Arc;

use crate::prelude::{Material, MaterialInstance};
use crate::{assets::material::AlphaMode, prelude::GpuMateiral};

#[derive(Debug, Clone)]
pub struct PbrMaterial {
    pub base_color_factor: Color,
    pub base_color_texture: Option<AssetHandle<Texture>>,
    pub metallic_factor: f32,
    pub roughness_factor: f32,
    pub metallic_roughness_texture: Option<AssetHandle<Texture>>,
    pub normal_scale: f32,
    pub normal_texture: Option<AssetHandle<Texture>>,
    pub ambient_occlusion_strength: f32,
    pub occlusion_texture: Option<AssetHandle<Texture>>,
    pub emissive_factor: Color,
    pub emissive_texture: Option<AssetHandle<Texture>>,
    pub texture_scale: math::Vec2,
    pub double_sided: bool,
    pub alpha_mode: AlphaMode,
    pub alpha_cutoff: f32,
    pub cast_shadows: bool,
}

impl Default for PbrMaterial {
    fn default() -> Self {
        Self {
            base_color_factor: Color::WHITE,
            base_color_texture: None,
            metallic_factor: 0.0,
            roughness_factor: 0.5,
            metallic_roughness_texture: None,
            normal_scale: 1.0,
            normal_texture: None,
            ambient_occlusion_strength: 1.0,
            occlusion_texture: None,
            emissive_factor: Color::BLACK,
            emissive_texture: None,
            texture_scale: Vec2::ONE,
            double_sided: false,
            alpha_mode: AlphaMode::Opaque,
            alpha_cutoff: 0.5,
            cast_shadows: true,
        }
    }
}

impl PbrMaterial {
    /// Base color factor (vec4)
    pub fn with_base_color_factor(mut self, base_color_factor: impl Into<Color>) -> Self {
        self.base_color_factor = base_color_factor.into();
        self
    }

    pub fn base_color_factor(&self) -> Color {
        self.base_color_factor
    }

    /// Base color texture
    pub fn with_base_color_texture(mut self, texture: AssetHandle<Texture>) -> Self {
        self.base_color_texture = Some(texture);
        self
    }

    pub fn base_color_texture(&self) -> Option<AssetHandle<Texture>> {
        self.base_color_texture.clone()
    }

    /// Metallic factor
    pub fn with_metallic_factor(mut self, metallic_factor: f32) -> Self {
        self.metallic_factor = metallic_factor;
        self
    }

    pub fn metallic_factor(&self) -> f32 {
        self.metallic_factor
    }

    /// Roughness factor
    pub fn with_roughness_factor(mut self, roughness_factor: f32) -> Self {
        self.roughness_factor = roughness_factor;
        self
    }

    pub fn roughness_factor(&self) -> f32 {
        self.roughness_factor
    }

    /// Metallic/Roughness texture
    pub fn with_metallic_roughness_texture(mut self, texture: AssetHandle<Texture>) -> Self {
        self.metallic_roughness_texture = Some(texture);
        self
    }

    pub fn metallic_roughness_texture(&self) -> Option<AssetHandle<Texture>> {
        self.metallic_roughness_texture.clone()
    }

    /// Normal scale
    pub fn with_normal_scale(mut self, normal_scale: f32) -> Self {
        self.normal_scale = normal_scale;
        self
    }

    pub fn normal_scale(&self) -> f32 {
        self.normal_scale
    }

    /// Normal texture
    pub fn with_normal_texture(mut self, texture: AssetHandle<Texture>) -> Self {
        self.normal_texture = Some(texture);
        self
    }

    pub fn normal_texture(&self) -> Option<AssetHandle<Texture>> {
        self.normal_texture.clone()
    }

    /// Ambient occlusion strength
    pub fn with_ambient_occlusion_strength(mut self, strength: f32) -> Self {
        self.ambient_occlusion_strength = strength;
        self
    }

    pub fn ambient_occlusion_strength(&self) -> f32 {
        self.ambient_occlusion_strength
    }

    /// Occlusion texture
    pub fn with_occlusion_texture(mut self, texture: AssetHandle<Texture>) -> Self {
        self.occlusion_texture = Some(texture);
        self
    }

    pub fn occlusion_texture(&self) -> Option<AssetHandle<Texture>> {
        self.occlusion_texture.clone()
    }

    /// Emissive factor
    pub fn with_emissive_factor(mut self, emissive_factor: Color) -> Self {
        self.emissive_factor = emissive_factor;
        self
    }

    pub fn emissive_factor(&self) -> Color {
        self.emissive_factor
    }

    /// Emissive texture
    pub fn with_emissive_texture(mut self, texture: AssetHandle<Texture>) -> Self {
        self.emissive_texture = Some(texture);
        self
    }

    pub fn emissive_texture(&self) -> Option<AssetHandle<Texture>> {
        self.emissive_texture.clone()
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
        self
    }

    pub fn with_shadows(mut self, casts_shadows: bool) -> Self {
        self.cast_shadows = casts_shadows;
        self
    }

    pub fn alpha_cutoff(&self) -> f32 {
        self.alpha_cutoff
    }

    /// Sets the texture/UV scale for all textures.
    ///
    /// This allows you to scale texture coordinates without modifying vertex data.
    /// Useful for tiling textures or adjusting texture density.
    ///
    /// # Arguments
    /// - `scale` - The scale factor (Vec2). Default is (1.0, 1.0).
    ///
    /// # Example
    /// ```rust,ignore
    /// // Tile the texture 2x horizontally and 3x vertically
    /// material.with_texture_scale(math::vec2(2.0, 3.0))
    /// ```
    pub fn with_texture_scale(mut self, scale: impl Into<math::Vec2>) -> Self {
        self.texture_scale = scale.into();
        self
    }

    pub fn texture_scale(&self) -> math::Vec2 {
        self.texture_scale
    }
}

impl IntoAsset<Material> for PbrMaterial {
    fn into_asset(
        self,
        _loader: &<Material as maple_engine::asset::Asset>::Loader,
        _library: &AssetLibrary, // no sub assets
    ) -> Result<Material, maple_engine::asset::LoadErr> {
        Ok(Material::new(self))
    }
}

pub struct GpuPbrMaterial {
    uniform: Buffer<MaterialBufferData>,
    descriptor: DescriptorSet,
}

impl GpuMateiral for GpuPbrMaterial {
    fn descriptor_set(&self) -> DescriptorSet {
        self.descriptor.clone()
    }
}

impl MaterialInstance for PbrMaterial {
    fn vertex_shader() -> maple_renderer::shader_asset::ShaderSource {
        include_str!("../../../res/shaders/default/default.vert.wgsl").into()
    }

    fn fragment_shader() -> maple_renderer::shader_asset::ShaderSource {
        include_str!("../../../res/shaders/default/default.frag.wgsl").into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    fn casts_shadows(&self) -> bool {
        self.cast_shadows
    }

    fn layout(&self, rcx: &RenderContext) -> DescriptorSetLayout {
        rcx.get_or_create_layout(DescriptorSetLayoutDescriptor {
            label: Some("pbr_material_layout"),
            visibility: StageFlags::VERTEX | StageFlags::FRAGMENT,
            layout: &[
                DescriptorBindingType::UniformBuffer,
                // base color
                DescriptorBindingType::TextureView { filterable: true },
                DescriptorBindingType::Sampler { filtering: true },
                // metallic roughness
                DescriptorBindingType::TextureView { filterable: true },
                DescriptorBindingType::Sampler { filtering: true },
                // ambient occlusion
                DescriptorBindingType::TextureView { filterable: true },
                DescriptorBindingType::Sampler { filtering: true },
                // emissive
                DescriptorBindingType::TextureView { filterable: true },
                DescriptorBindingType::Sampler { filtering: true },
                // normal
                DescriptorBindingType::TextureView { filterable: true },
                DescriptorBindingType::Sampler { filtering: true },
            ],
        })
    }

    fn prepare(
        &self,
        rcx: &RenderContext,
        assets: &AssetLibrary,
        layout: &DescriptorSetLayout,
    ) -> Option<Arc<dyn GpuMateiral + 'static>> {
        let defaults = rcx.get_default_texture();

        // If the texture isn't loaded yet, returns None; otherwise returns the
        // loaded texture, the default texture, or an error texture on load failure.
        let resolve_texture =
            |handle: &Option<AssetHandle<Texture>>, fallback: &Texture| -> Option<Texture> {
                match handle {
                    None => Some(fallback.clone()),
                    Some(h) => assets.get(h).map(|asset| asset.clone()),
                }
            };

        let slots = [
            (&self.base_color_texture, &defaults.white),
            (&self.metallic_roughness_texture, &defaults.white),
            (&self.occlusion_texture, &defaults.white),
            (&self.emissive_texture, &defaults.white),
            (&self.normal_texture, &defaults.normal),
        ];

        let resolved: Option<Vec<Texture>> = slots
            .iter()
            .map(|(handle, fallback)| resolve_texture(handle, fallback))
            .collect();

        let Some(resolved) = resolved else {
            return None;
        };
        let [base_color, metallic_roughness, occlusion, emissive, normal]: [Texture; 5] =
            resolved.try_into().unwrap();

        let uniform = self.get_buffer();
        let uniform_buffer = rcx.device().create_uniform_buffer(&uniform);

        let descriptor = rcx.device().build_descriptor_set(
            DescriptorSet::builder(&layout)
                .uniform(0, &uniform_buffer)
                .texture_view(1, &base_color.create_view())
                .sampler(2, &defaults.sampler)
                .texture_view(3, &metallic_roughness.create_view())
                .sampler(4, &defaults.sampler)
                .texture_view(5, &occlusion.create_view())
                .sampler(6, &defaults.sampler)
                .texture_view(7, &emissive.create_view())
                .sampler(8, &defaults.sampler)
                .texture_view(9, &normal.create_view())
                .sampler(10, &defaults.sampler),
        );

        Some(Arc::new(GpuPbrMaterial {
            uniform: uniform_buffer,
            descriptor: descriptor,
        }))
    }

    fn update(&self, rcx: &RenderContext, gpu: &dyn GpuMateiral) {
        let Some(gpu_material) = gpu.as_any().downcast_ref::<GpuPbrMaterial>() else {
            return;
        };

        rcx.queue()
            .write_buffer(&gpu_material.uniform, &self.get_buffer());
    }
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
    pub parallax_scale: f32,
    pub alpha_mode: u32,         // 0 opaque, 1 mask, 2 blend
    pub unlit: u32,              // 0 lit, 1 unlit
    pub texture_scale: [f32; 2], // UV scale for all textures
    _padding: [f32; 2],          // Padding for alignment
}

impl PbrMaterial {
    fn get_buffer(&self) -> MaterialBufferData {
        MaterialBufferData {
            base_color_factor: self.base_color_factor.into(),
            metallic_factor: self.metallic_factor,
            roughness_factor: self.roughness_factor,
            normal_scale: self.normal_scale,
            ambient_occlusion_strength: self.ambient_occlusion_strength,
            emissive_factor: self.emissive_factor.into(),
            texture_scale: self.texture_scale.into(),
            alpha_cutoff: self.alpha_cutoff,
            parallax_scale: 1.0,
            alpha_mode: match self.alpha_mode {
                AlphaMode::Opaque => 0u32,
                AlphaMode::Mask => 1u32,
                AlphaMode::Blend => 2u32,
            },
            unlit: 0,
            _padding: [0.0, 0.0],
        }
    }
}
