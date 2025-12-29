use std::sync::OnceLock;

use wgpu::{Device, Queue};

use crate::core::texture::{
    FilterMode, Sampler, SamplerOptions, Texture, TextureCreateInfo, TextureMode, TextureUsage,
};

pub struct DefaultTexture {
    pub white: Texture,
    pub normal: Texture,
    pub sampler: Sampler,
}

static DEFAULT_TEXTURES: OnceLock<DefaultTexture> = OnceLock::new();

impl DefaultTexture {
    pub(crate) fn get(device: &Device, queue: &Queue) -> &'static DefaultTexture {
        DEFAULT_TEXTURES.get_or_init(|| {
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
            white.write(queue, &[255, 255, 255, 255]);

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
            normal.write(queue, &[128, 128, 255, 255]);

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

            DefaultTexture {
                white,
                normal,
                sampler,
            }
        })
    }
}
