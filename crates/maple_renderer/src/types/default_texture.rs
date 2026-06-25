use std::sync::Arc;

use wgpu::{Device, Queue};

use crate::core::texture::{
    FilterMode, Sampler, SamplerOptions, Texture, TextureCreateInfo, TextureCube,
    TextureCubeCreateInfo, TextureFormat, TextureMode, TextureUsage,
};

pub struct DefaultTexture {
    pub white: Arc<Texture>,
    pub normal: Arc<Texture>,
    pub error: Arc<Texture>,
    pub sampler: Sampler,
    // IBL defaults - black textures so objects reflect nothing
    pub irradiance_cubemap: TextureCube,
    pub prefilter_cubemap: TextureCube,
    pub brdf_lut: Texture,
}

impl DefaultTexture {
    pub(crate) fn init_textures(device: &Device, queue: &Queue) -> DefaultTexture {
        let white = Texture::create(
            device,
            &TextureCreateInfo {
                label: Some("Default White"),
                width: 1,
                height: 1,
                format: crate::core::texture::TextureFormat::RGBA8,
                usage: TextureUsage::TEXTURE_BINDING | TextureUsage::COPY_DST,
                sample_count: 1,
                mip_level: 1,
            },
        );
        white.write(queue, &[255u8, 255u8, 255u8, 255u8]);

        let normal = Texture::create(
            device,
            &TextureCreateInfo {
                label: Some("Default Normal"),
                width: 1,
                height: 1,
                format: crate::core::texture::TextureFormat::RGBA8,
                usage: TextureUsage::TEXTURE_BINDING | TextureUsage::COPY_DST,
                sample_count: 1,
                mip_level: 1,
            },
        );
        normal.write(queue, &[128u8, 128u8, 255u8, 255u8]);

        let sampler = Texture::create_sampler(
            device,
            SamplerOptions {
                mode_u: TextureMode::Repeat,
                mode_v: TextureMode::Repeat,
                mode_w: TextureMode::Repeat,
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Linear,
                compare: None,
            },
        );

        // Create default black IBL textures
        // These ensure objects reflect nothing when no environment is present
        let irradiance_cubemap = TextureCube::create(
            device,
            &TextureCubeCreateInfo {
                label: Some("Default Irradiance Cubemap"),
                size: 1,
                format: TextureFormat::RGBA16Float,
                usage: TextureUsage::TEXTURE_BINDING | TextureUsage::COPY_DST,
                mip_level: 1,
            },
        );

        let prefilter_cubemap = TextureCube::create(
            device,
            &TextureCubeCreateInfo {
                label: Some("Default Prefilter Cubemap"),
                size: 1,
                format: TextureFormat::RGBA16Float,
                usage: TextureUsage::TEXTURE_BINDING | TextureUsage::COPY_DST,
                mip_level: 1,
            },
        );

        // Create 2x2 magenta/black checkerboard
        let error_pixels = vec![
            255u8, 0u8, 255u8, 255u8, // Magenta
            0u8, 0u8, 0u8, 255u8, // Black
            0u8, 0u8, 0u8, 255u8, // Black
            255u8, 0u8, 255u8, 255u8, // Magenta
        ];

        let error = Texture::create(
            device,
            &TextureCreateInfo {
                label: Some("error_texture"),
                width: 2,
                height: 2,
                format: TextureFormat::RGBA8,
                usage: TextureUsage::TEXTURE_BINDING | TextureUsage::COPY_DST,
                mip_level: 1,
                sample_count: 1,
            },
        );

        error.write(queue, &error_pixels);

        // BRDF LUT - 1x1 with (0.0, 0.0) means no specular contribution
        let brdf_lut = Texture::create(
            device,
            &TextureCreateInfo {
                label: Some("Default BRDF LUT"),
                width: 1,
                height: 1,
                format: TextureFormat::RG32Float,
                usage: TextureUsage::TEXTURE_BINDING | TextureUsage::COPY_DST,
                sample_count: 1,
                mip_level: 1,
            },
        );
        // Write (0.0, 0.0) as 8 bytes (two f32s)
        brdf_lut.write(queue, &[0u8; 8]);

        DefaultTexture {
            white: Arc::new(white),
            normal: Arc::new(normal),
            sampler,
            error: Arc::new(error),
            irradiance_cubemap,
            prefilter_cubemap,
            brdf_lut,
        }
    }
}
