use std::default;

use bitflags::bitflags;
use wgpu::{
    AddressMode, Device, Origin3d, Queue, TexelCopyBufferLayout, TexelCopyTextureInfo,
    TextureAspect, TextureDescriptor, TextureDimension, TextureUsages, TextureViewDescriptor,
};
pub struct Texture {
    pub(crate) inner: wgpu::Texture,
    width: u32,
    height: u32,
    format: TextureFormat,
}

pub struct TextureView {
    pub(crate) inner: wgpu::TextureView,
}

pub struct Sampler {
    pub(crate) inner: wgpu::Sampler,
}

pub struct SamplerOptions {
    mode_u: TextureMode,
    mode_v: TextureMode,
    mode_w: TextureMode,
    mag_filter: FilterMode,
    min_filter: FilterMode,
}

impl From<SamplerOptions> for wgpu::SamplerDescriptor<'static> {
    fn from(value: SamplerOptions) -> Self {
        Self {
            address_mode_u: value.mode_u.into(),
            address_mode_v: value.mode_v.into(),
            address_mode_w: value.mode_w.into(),
            mag_filter: value.mag_filter.into(),
            min_filter: value.min_filter.into(),
            ..Default::default()
        }
    }
}

/// how its sampled when uv is outside of texture
pub enum TextureMode {
    ClampToEdge,
    Repeat,
    MirrorRepeat,
}

impl From<TextureMode> for AddressMode {
    fn from(value: TextureMode) -> Self {
        match value {
            TextureMode::ClampToEdge => Self::ClampToEdge,
            TextureMode::Repeat => Self::Repeat,
            TextureMode::MirrorRepeat => Self::MirrorRepeat,
        }
    }
}

/// how its sampled when uv is between 2 texels
pub enum FilterMode {
    Linear,
    Nearest,
}

impl From<FilterMode> for wgpu::FilterMode {
    fn from(value: FilterMode) -> Self {
        match value {
            FilterMode::Linear => Self::Linear,
            FilterMode::Nearest => Self::Nearest,
        }
    }
}

#[derive(Clone, Debug)]
pub enum TextureFormat {
    RGBA8,
    RGBA16,
    R8,
    R16,
}

impl TextureFormat {
    pub fn byte_offset(&self) -> u32 {
        match self {
            Self::RGBA8 => 4,
            Self::RGBA16 => 8,
            Self::R8 => 1,
            Self::R16 => 2,
        }
    }
}

impl From<TextureFormat> for wgpu::TextureFormat {
    fn from(value: TextureFormat) -> Self {
        match value {
            TextureFormat::RGBA8 => Self::Rgba8Unorm,
            TextureFormat::RGBA16 => Self::Rgba16Unorm,
            TextureFormat::R8 => Self::R8Unorm,
            TextureFormat::R16 => Self::R16Unorm,
        }
    }
}

bitflags! {
    pub struct TextureUsage: u32 {
        const COPY_SRC = 1 << 0;
        const COPY_DST = 1 << 1;
        const RENDER_ATTACHMENT = 1 << 2;
        const TEXTURE_BINDING = 1 << 3;
    }
}

impl From<TextureUsage> for wgpu::TextureUsages {
    fn from(value: TextureUsage) -> Self {
        let mut usage = Self::empty();
        if value.contains(TextureUsage::COPY_SRC) {
            usage |= TextureUsages::COPY_SRC;
        }
        if value.contains(TextureUsage::COPY_DST) {
            usage |= TextureUsages::COPY_DST;
        }
        if value.contains(TextureUsage::RENDER_ATTACHMENT) {
            usage |= TextureUsages::RENDER_ATTACHMENT;
        }
        if value.contains(TextureUsage::TEXTURE_BINDING) {
            usage |= TextureUsages::TEXTURE_BINDING;
        }
        usage
    }
}

pub struct TextureCreateInfo {
    label: Option<&'static str>,
    width: u32,
    height: u32,
    format: TextureFormat,
    usage: TextureUsage,
}

impl Texture {
    pub fn create(device: Device, info: TextureCreateInfo) -> Self {
        let texture_size = wgpu::Extent3d {
            width: info.width,
            height: info.height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
            label: info.label,
            size: texture_size,
            format: info.format.clone().into(),
            usage: info.usage.into(),
            dimension: TextureDimension::D2,
            mip_level_count: 1,
            sample_count: 1,
            view_formats: &[],
        });

        Self {
            inner: texture,
            height: info.height,
            width: info.height,
            format: info.format,
        }
    }

    pub fn write(&self, queue: &Queue, data: &[u8]) {
        let size = wgpu::Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: 1,
        };

        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &self.inner,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            data,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(self.format.byte_offset() * self.width),
                rows_per_image: Some(self.height),
            },
            size,
        );
    }

    pub fn create_view(&self) -> TextureView {
        let view = self.inner.create_view(&TextureViewDescriptor::default());
        TextureView { inner: view }
    }

    pub fn create_sampler(device: &Device, options: SamplerOptions) -> Sampler {
        let sampler = device.create_sampler(&options.into());
        Sampler { inner: sampler }
    }
}
