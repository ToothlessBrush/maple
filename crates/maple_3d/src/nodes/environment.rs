use maple_engine::{
    Node,
    asset::{AssetHandle, AssetLibrary},
    prelude::NodeTransform,
};
use maple_renderer::core::{
    RenderContext,
    texture::{LazyTexture, Texture},
};

/// Resolution scale factor for environment maps
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionScale {
    /// Use full source resolution (default)
    Full,
    /// Use half resolution (1/2)
    Half,
    /// Use quarter resolution (1/4)
    Quarter,
    /// Use eighth resolution (1/8)
    Eighth,
}

impl ResolutionScale {
    /// Apply the scale to a resolution value
    pub fn apply(&self, resolution: u32) -> u32 {
        match self {
            ResolutionScale::Full => resolution,
            ResolutionScale::Half => resolution / 2,
            ResolutionScale::Quarter => resolution / 4,
            ResolutionScale::Eighth => resolution / 8,
        }
    }
}

pub struct Environment {
    pub transform: NodeTransform,

    hdri_source: AssetHandle<LazyTexture>,
    ibl_strength: f32,

    cubemap_scale: ResolutionScale,
    irradiance_resolution: u32,
    prefilter_resolution: u32,
    brdf_resolution: u32,
}

impl Node for Environment {
    fn get_transform(&mut self) -> &mut NodeTransform {
        &mut self.transform
    }
}

impl Environment {
    pub fn new(hdr: AssetHandle<LazyTexture>) -> Self {
        // Automatically determine base resolution from source HDR dimensions
        // For equirectangular maps, width is typically 2x height, so we use height
        // as the base cubemap resolution

        // most of this is handled by the rendergraph
        Self {
            transform: NodeTransform::default(),
            hdri_source: hdr,
            ibl_strength: 1.0, // Default strength
            cubemap_scale: ResolutionScale::Full,
            irradiance_resolution: 32,
            prefilter_resolution: 128,
            brdf_resolution: 512,
        }
    }

    pub fn get_hdri_texture(&self, rcx: &RenderContext, assets: &AssetLibrary) -> Option<Texture> {
        assets
            .get::<LazyTexture>(&self.hdri_source)
            .asset()
            .map(|texture| texture.texture(rcx))
    }

    pub fn ibl_strength(&self) -> f32 {
        self.ibl_strength
    }

    pub fn set_ibl_strength(&mut self, strength: f32) {
        self.ibl_strength = strength;
    }

    pub fn with_ibl_strength(mut self, strength: f32) -> Self {
        self.ibl_strength = strength;
        self
    }

    /// Set the resolution scale for the cubemap
    pub fn with_resolution_scale(mut self, scale: ResolutionScale) -> Self {
        self.cubemap_scale = scale;
        self
    }

    /// Get the actual cubemap resolution after applying scale
    pub fn get_cubemap_resolution(
        &self,
        rcx: &RenderContext,
        assets: &AssetLibrary,
    ) -> Option<u32> {
        // Get the HDRI texture to determine base resolution
        let texture = self.get_hdri_texture(rcx, assets)?;

        // For equirectangular maps, width is typically 2x height
        // Use height as the base cubemap resolution
        let base_resolution = texture.height();

        // Apply the resolution scale factor
        Some(self.cubemap_scale.apply(base_resolution))
    }

    /// Set custom irradiance map resolution
    pub fn with_irradiance_resolution(mut self, resolution: u32) -> Self {
        self.irradiance_resolution = resolution;
        self
    }

    /// Set custom prefilter map resolution
    pub fn with_prefilter_resolution(mut self, resolution: u32) -> Self {
        self.prefilter_resolution = resolution;
        self
    }

    /// Set custom BRDF LUT resolution
    pub fn with_brdf_resolution(mut self, resolution: u32) -> Self {
        self.brdf_resolution = resolution;
        self
    }

    /// Quality preset: Low (quarter resolution, reduced IBL quality)
    /// Good for low-end hardware or mobile
    pub fn quality_low(mut self) -> Self {
        self.cubemap_scale = ResolutionScale::Quarter;
        self.irradiance_resolution = 16;
        self.prefilter_resolution = 64;
        self.brdf_resolution = 256;
        self
    }

    /// Quality preset: Medium (half resolution)
    /// Balanced quality and performance
    pub fn quality_medium(mut self) -> Self {
        self.cubemap_scale = ResolutionScale::Half;
        self.irradiance_resolution = 32;
        self.prefilter_resolution = 128;
        self.brdf_resolution = 512;
        self
    }

    /// Quality preset: High (full resolution)
    /// Best quality, default settings
    pub fn quality_high(mut self) -> Self {
        self.cubemap_scale = ResolutionScale::Full;
        self.irradiance_resolution = 32;
        self.prefilter_resolution = 128;
        self.brdf_resolution = 512;
        self
    }

    // /// Get the actual cubemap resolution after applying scale
    // pub fn get_cubemap_resolution(&self) -> u32 {
    //     self.cubemap_scale.apply(self.cubemap_base_resolution)
    // }

    pub fn get_irradiance_resolution(&self) -> u32 {
        self.irradiance_resolution
    }

    pub fn get_prefilter_resolution(&self) -> u32 {
        self.prefilter_resolution
    }

    pub fn get_brdf_resolution(&self) -> u32 {
        self.brdf_resolution
    }

    pub fn get_resolution_scale(&self) -> ResolutionScale {
        self.cubemap_scale
    }
}
