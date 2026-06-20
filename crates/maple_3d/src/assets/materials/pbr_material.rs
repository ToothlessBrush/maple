use bytemuck::{Pod, Zeroable};
use glam::{self as math, Vec4};
use maple_engine::{
    asset::{AssetHandle, AssetLibrary, AssetState, IntoAsset},
    utils::Color,
};
use maple_renderer::core::{
    Buffer, DescriptorBindingType, DescriptorSet, DescriptorSetLayout,
    DescriptorSetLayoutDescriptor, LazyBuffer, LazyBufferable, RenderContext, RenderDevice,
    RenderQueue, StageFlags,
    texture::{LazyTexture, Sampler, Texture},
};

use std::{cell::OnceCell, sync::Arc};

use crate::prelude::{Material, MaterialDescriptorState, MaterialInstance, PipelineStage};

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
        loader: &<Material as maple_engine::asset::Asset>::Loader,
        library: &AssetLibrary,
    ) -> Result<Arc<Material>, maple_engine::asset::LoadErr> {
        Material::new(PbrMaterialInstance {})
    }
}

/// Material properties for the mesh
#[derive(Clone)]
pub struct PbrMaterialInstance {
    /// Base color factor of the material
    base_color_factor: math::Vec4,
    /// texture for base color
    base_color_texture: Option<AssetHandle<Texture>>,
    base_color_sampler: Option<Sampler>,

    /// Metallic factor of the material
    metallic_factor: f32,
    /// Roughness factor of the material
    roughness_factor: f32,
    /// texture for materials metallic roughness
    ///
    /// metallic on blue channel and roughness on green channel
    metallic_roughness_texture: Option<AssetHandle<Texture>>,
    metallic_roughness_sampler: Option<Sampler>,

    /// scale of objects normals
    normal_scale: f32,
    /// texture for normals
    normal_texture: Option<AssetHandle<Texture>>,
    normal_sampler: Option<Sampler>,

    /// strength of ambient occlusion
    ambient_occlusion_strength: f32,
    /// texture for ambient occlusion
    occlusion_texture: Option<AssetHandle<Texture>>,
    occlusion_sampler: Option<Sampler>,

    /// strength of an objects emission
    emissive_factor: math::Vec3,
    /// texture for emission
    emissive_texture: Option<AssetHandle<Texture>>,
    emissive_sampler: Option<Sampler>,

    // depth mapping
    parallax_scale: f32,
    parallax_texture: Option<AssetHandle<Texture>>,
    parallax_sampler: Option<Sampler>,

    /// UV/Texture scale for all textures
    texture_scale: math::Vec2,

    /// Double sided property of the material
    double_sided: bool,
    /// Alpha mode of the material
    alpha_mode: AlphaMode,
    /// Alpha cutoff of the material
    alpha_cutoff: f32,
    /// Unlit material (no lighting calculations)
    unlit: bool,

    buffer_data: MaterialBufferData,

    uniform: Buffer<MaterialBufferData>,

    descriptor: Arc<OnceCell<DescriptorSet>>,
}

impl MaterialInstance for PbrMaterialInstance {
    fn stage(&self) -> crate::prelude::PipelineStage {
        match self.alpha_mode {
            AlphaMode::Opaque | AlphaMode::Mask => PipelineStage::Opaque,
            AlphaMode::Blend => PipelineStage::Transparent,
        }
    }

    fn layout(&self, rcx: &RenderContext) -> DescriptorSetLayout {
        rcx.get_or_create_layout(DescriptorSetLayoutDescriptor {
            label: Some("pbr_material_layout"),
            visibility: StageFlags::FRAGMENT,
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
                DescriptorBindingType::TextureView { filterable: true },
                DescriptorBindingType::Sampler { filtering: true },
            ],
        })
    }

    fn pipeline(
        &self,
        rcx: &RenderContext,
        pipeline_layout: maple_renderer::core::PipelineLayout,
    ) -> maple_renderer::core::RenderPipeline {
        rcx.device()
            .create_pipeline(maple_renderer::core::PipelineCreateInfo {
                label: Some("Pbr Material Pipeline"),
                layout: pipeline_layout,
                shader: ,
                color_formats: (),
                depth: (),
                cull_mode: (),
                alpha_mode: (),
                sample_count: (),
                use_vertex_buffer: (),
            })
    }

    fn descriptor_set(
        &mut self,
        assets: &AssetLibrary,
        rcx: &RenderContext,
        layout: &DescriptorSetLayout,
    ) -> MaterialDescriptorState {
        self.update_buffer(rcx.queue());

        // Fast path: already cached.
        if let Some(set) = self.descriptor.get() {
            return MaterialDescriptorState::Ready(set.clone());
        }

        let defaults = rcx.get_default_texture();

        // If the texture isn't loaded yet, returns None; otherwise returns the
        // loaded texture, the default texture, or an error texture on load failure.
        let resolve_texture = |handle: &Option<AssetHandle<Texture>>,
                               fallback: &Arc<Texture>|
         -> Option<Arc<Texture>> {
            match handle {
                None => Some(fallback.clone()),
                Some(h) => match assets.get::<Texture>(h) {
                    AssetState::Loaded(asset) => Some(asset), // already Arc<Texture>
                    AssetState::Error(_) => Some(defaults.error.clone()),
                    AssetState::Loading => None,
                },
            }
        };

        let slots = [
            (&self.base_color_texture, &defaults.white),
            (&self.metallic_roughness_texture, &defaults.white),
            (&self.occlusion_texture, &defaults.white),
            (&self.emissive_texture, &defaults.white),
            (&self.normal_texture, &defaults.normal),
            (&self.parallax_texture, &defaults.white),
        ];

        let resolved: Option<Vec<Arc<Texture>>> = slots
            .iter()
            .map(|(handle, fallback)| resolve_texture(handle, fallback))
            .collect();

        let Some(resolved) = resolved else {
            return MaterialDescriptorState::Loading;
        };
        let [
            base_color,
            metallic_roughness,
            occlusion,
            emissive,
            normal,
            parallax,
        ]: [Arc<Texture>; 6] = resolved.try_into().unwrap();

        let set = self.descriptor.get_or_init(|| {
            rcx.device().build_descriptor_set(
                DescriptorSet::builder(&layout)
                    .uniform(0, &self.uniform)
                    .texture_view(1, &base_color.create_view())
                    .sampler(
                        2,
                        self.base_color_sampler
                            .as_ref()
                            .unwrap_or(&defaults.sampler),
                    )
                    .texture_view(3, &metallic_roughness.create_view())
                    .sampler(
                        4,
                        self.metallic_roughness_sampler
                            .as_ref()
                            .unwrap_or(&defaults.sampler),
                    )
                    .texture_view(5, &occlusion.create_view())
                    .sampler(
                        6,
                        self.occlusion_sampler.as_ref().unwrap_or(&defaults.sampler),
                    )
                    .texture_view(7, &emissive.create_view())
                    .sampler(
                        8,
                        self.emissive_sampler.as_ref().unwrap_or(&defaults.sampler),
                    )
                    .texture_view(9, &normal.create_view())
                    .sampler(
                        10,
                        self.normal_sampler.as_ref().unwrap_or(&defaults.sampler),
                    )
                    .texture_view(11, &parallax.create_view())
                    .sampler(
                        12,
                        self.parallax_sampler.as_ref().unwrap_or(&defaults.sampler),
                    ),
            )
        });

        MaterialDescriptorState::Ready(set.clone())
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

impl PbrMaterialInstance {
    /// Update the internal buffer and write to the GPU
    fn update_buffer(&mut self, queue: &RenderQueue) {
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
            parallax_scale: self.parallax_scale,
            alpha_mode: match self.alpha_mode {
                AlphaMode::Opaque => 0u32,
                AlphaMode::Mask => 1u32,
                AlphaMode::Blend => 2u32,
            },
            unlit: if self.unlit { 1u32 } else { 0u32 },
            texture_scale: self.texture_scale.into(),
            _padding: [0.0, 0.0],
        };
        queue.write_buffer(&self.uniform, &self.buffer_data);
    }
}
