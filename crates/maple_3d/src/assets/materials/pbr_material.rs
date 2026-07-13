use bytemuck::{Pod, Zeroable};
use glam::{self as math, Vec2};
use maple_engine::{
    asset::{AssetHandle, AssetLibrary, AssetStatus, IntoAsset},
    color::Color,
};
use maple_renderer::core::{
    Buffer, CullMode, DescriptorBindingType, DescriptorSet, DescriptorSetLayout,
    DescriptorSetLayoutDescriptor, RenderContext, StageFlags, texture::Texture,
};

use std::sync::Arc;

use crate::assets::material::{AlphaMode, GpuMateiral};
use crate::prelude::{Material, MaterialInstance};

/// Physically Based Rendered material
///
/// This material describes how a surface interacts with light in a realistic way by using
/// [`Self::metallic_factor`] and [`Self::roughness_factor`] based on glTF 2.0 metallic-roughness
/// model
///
/// This is the engines default material used and can be made directly by a [`Color`] or [`AssetHandle<Texture>`] through the asset system
///
/// # Example
/// ```ignore
/// let material = assets.add(PbrMaterial {
///     base_color_factor: Color::RED,
///     base_color_texture: Some(assets.load("res/2k_earth_daymap.jpg")),
///     metallic_factor: 1.0,
///     ..Default::default()
/// });
///
/// // `Color` and `AssetHanle<Texture>` convert into `PbrMaterial`
/// let color_material = assets.add(Color::RED);
/// let texture_material = assets.add(assets.load("res/2k_earth_daymap.jpg"));
/// ```
#[derive(Debug, Clone)]
pub struct PbrMaterial {
    /// the color the material appears
    ///
    /// the alpha channel meaning depends on [`Self::alpha_mode`]
    /// - [`AlphaMode::Opaque`] : ignored
    /// - [`AlphaMode::Mask`] : discarded if alpha < [`Self::alpha_cutoff`]
    /// - [`AlphaMode::Blend`] : used as transparency factor
    ///
    ///
    /// multiplied with [`Self::base_color_texture`]: `base_color_factor * base_color_texture`
    ///
    /// Default: [`Color::WHITE`]
    pub base_color_factor: Color,

    /// Texture used for the base color of the material. see [`Self::base_color_factor`]
    ///
    /// Default: [`Option::None`]
    pub base_color_texture: Option<AssetHandle<Texture>>,

    /// How metallic this material appears
    ///
    /// value is between `0.0` and `1.0` where 1.0 is fully metallic
    ///
    /// Default: `0.0`
    pub metallic_factor: f32,

    /// how rough this material appears which affects how uniformly the light reflects back to the
    /// camera
    ///
    /// value is between `0.0` and `1.0` where 1.0 is fully rough
    ///
    /// Default: `0.5`
    pub roughness_factor: f32,

    /// texture used for metallic and roughness multipied by the factors for each
    ///
    /// metallic is on blue channel and roughness is on green.
    ///
    /// Default: [`Option::None`]
    pub metallic_roughness_texture: Option<AssetHandle<Texture>>,

    /// scale the object normals are scaled by usually unused
    ///
    /// Default: `1.0`
    pub normal_scale: f32,

    /// texture used within the shader for the surface normal which can create an illusion of detail
    /// without the need for a more complex mesh
    ///
    /// when a light ray hits an object its reflected around the normal which is perpendicular to the
    /// surface plane
    ///
    /// Default: [`Option::None`]
    pub normal_texture: Option<AssetHandle<Texture>>,

    /// how much ambient light this material is exposed to
    ///
    /// more occluded sections may appear darker so this provides a almost free way for more light
    /// detail
    ///
    /// Default: `1.0`
    pub ambient_occlusion_strength: f32,

    /// texture used for the ambient occlusion of a material see [`Self::ambient_occlusion_strength`]
    ///
    /// Default: [`Option::None`]
    pub occlusion_texture: Option<AssetHandle<Texture>>,

    /// Color that it emitted to the camera
    ///
    /// this color is added to the materials output color after lighting calculations.
    ///
    /// Default: [`Color::BLACK`]
    pub emissive_factor: Color,

    /// Texture used for material Emission. see [`Self::emissive_factor`]
    ///
    /// Default: [`Option::None`]
    pub emissive_texture: Option<AssetHandle<Texture>>,

    /// the xy scale of the meshes texture cordinates.
    ///
    /// texture cords go from `0.0` to `1.0` textures repeat after 1.0 so scaling them can causing
    /// textures to tile or shrink.
    ///
    /// Default: [`Vec2::ONE`]
    pub texture_scale: math::Vec2,

    /// whether the objects renders both sides
    ///
    /// Default: `false`
    pub double_sided: bool,

    /// affects what the alpha channel of [`Self::base_color_factor`] does
    ///
    /// Default: [`AlphaMode::Opaque`]
    pub alpha_mode: AlphaMode,

    /// cutoff threshhold used if [`Self::alpha_mode`] is [`AlphaMode::Mask`]
    ///
    /// Default: `0.5`
    pub alpha_cutoff: f32,

    /// whether the material casts shadows or not
    ///
    /// Default: `true`
    pub cast_shadows: bool,

    /// which side of the material gets culled
    ///
    /// Default: [`CullMode::Back`]
    pub cull_mode: CullMode,
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
            cull_mode: CullMode::Back,
        }
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

impl IntoAsset<Material> for Color {
    fn into_asset(
        self,
        _loader: &<Material as maple_engine::asset::Asset>::Loader,
        _library: &AssetLibrary,
    ) -> Result<Material, maple_engine::asset::LoadErr> {
        Ok(Material::new(PbrMaterial {
            base_color_factor: self,
            ..Default::default()
        }))
    }
}

impl IntoAsset<Material> for AssetHandle<Texture> {
    fn into_asset(
        self,
        _loader: &<Material as maple_engine::asset::Asset>::Loader,
        _library: &AssetLibrary,
    ) -> Result<Material, maple_engine::asset::LoadErr> {
        Ok(Material::new(PbrMaterial {
            base_color_texture: Some(self),
            ..Default::default()
        }))
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
        include_str!("pbr.vert.wgsl").into()
    }

    fn fragment_shader() -> maple_renderer::shader_asset::ShaderSource {
        include_str!("pbr.frag.wgsl").into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    fn cull_mode(&self) -> CullMode {
        self.cull_mode
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
                    Some(h) => match assets.get_status(h) {
                        AssetStatus::Loaded(texture) => Some(texture.clone()),
                        AssetStatus::Error(_) => Some(defaults.error.clone()),
                        AssetStatus::Loading => None,
                        AssetStatus::Removed => Some(fallback.clone()),
                        _ => None,
                    },
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
