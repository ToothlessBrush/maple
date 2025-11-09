use bitflags::bitflags;
use image::imageops::FilterType::Triangle;
use wgpu::{
    AddressMode, Device, Origin3d, Queue, TexelCopyBufferLayout, TexelCopyTextureInfo,
    TextureAspect, TextureDescriptor, TextureDimension, TextureUsages, TextureViewDescriptor,
};

use crate::render_graph::graph::GraphResource;

pub struct TextureView {
    pub(crate) inner: wgpu::TextureView,
}

pub struct Sampler {
    pub(crate) inner: wgpu::Sampler,
}

pub struct SamplerOptions {
    pub mode_u: TextureMode,
    pub mode_v: TextureMode,
    pub mode_w: TextureMode,
    pub mag_filter: FilterMode,
    pub min_filter: FilterMode,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextureFormat {
    RGB8,
    RGB16,
    RGBA8,
    RGBA16,
    BGRA8,
    BGRA8Srgb,
    RGBA8Srgb,
    R8,
    R16,
    // depth format
    Depth32,
    Depth24,
    Depth24PlusStencil8,
}

impl TextureFormat {
    pub fn byte_offset(&self) -> u32 {
        match self {
            Self::RGBA8 => 4,
            Self::RGBA16 => 8,
            Self::R8 => 1,
            Self::R16 => 2,
            Self::RGB8 => 4,
            Self::RGB16 => 8,
            Self::BGRA8 => 4,
            Self::BGRA8Srgb => 4,
            Self::RGBA8Srgb => 4,
            Self::Depth32 | Self::Depth24 | Self::Depth24PlusStencil8 => 0,
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
            TextureFormat::RGB8 => Self::Rgba8Snorm,
            TextureFormat::RGB16 => Self::Rgba16Unorm,
            TextureFormat::BGRA8 => Self::Bgra8Unorm,
            TextureFormat::BGRA8Srgb => Self::Bgra8UnormSrgb,
            TextureFormat::RGBA8Srgb => Self::Rgba8UnormSrgb,
            TextureFormat::Depth32 => Self::Depth32Float,
            TextureFormat::Depth24 => Self::Depth24Plus,
            TextureFormat::Depth24PlusStencil8 => Self::Depth32FloatStencil8,
        }
    }
}

impl From<wgpu::TextureFormat> for TextureFormat {
    fn from(value: wgpu::TextureFormat) -> Self {
        match value {
            wgpu::TextureFormat::Rgba8Unorm => Self::RGBA8,
            wgpu::TextureFormat::Rgba16Unorm => Self::RGBA16,
            wgpu::TextureFormat::R8Unorm => Self::R8,
            wgpu::TextureFormat::R16Unorm => Self::R16,
            wgpu::TextureFormat::Rgba8Snorm => Self::RGB8,
            wgpu::TextureFormat::Bgra8Unorm => Self::BGRA8,
            wgpu::TextureFormat::Bgra8UnormSrgb => Self::BGRA8Srgb,
            wgpu::TextureFormat::Rgba8UnormSrgb => Self::RGBA8Srgb,
            wgpu::TextureFormat::Depth32Float => Self::Depth32,
            wgpu::TextureFormat::Depth24Plus => Self::Depth24,
            wgpu::TextureFormat::Depth32FloatStencil8 => Self::Depth24PlusStencil8,
            _ => panic!("Unsupported wgpu::TextureFormat: {:?}", value),
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
    pub label: Option<&'static str>,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub usage: TextureUsage,
}

#[derive(PartialEq, Eq)]
pub struct Texture {
    pub(crate) inner: wgpu::Texture,
    width: u32,
    height: u32,
    format: TextureFormat,
}

impl GraphResource for Texture {}

impl Texture {
    pub fn create(device: &Device, info: TextureCreateInfo) -> Self {
        let texture_size = wgpu::Extent3d {
            width: info.width,
            height: info.height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
            label: info.label,
            size: texture_size,
            format: info.format.into(),
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

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn write(&self, queue: &Queue, data: &[u8]) {
        let size = wgpu::Extent3d {
            width: self.width,
            height: self.height,
            depth_or_array_layers: 1,
        };

        if self.format == TextureFormat::RGB8 || self.format == TextureFormat::RGB16 {
            todo!("add padding for rgb format")
        }

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

    pub fn format(&self) -> TextureFormat {
        self.format
    }
}

pub struct TextureArrayCreateInfo {
    pub label: Option<&'static str>,
    pub width: u32,
    pub height: u32,
    pub array_layers: u32,
    pub format: TextureFormat,
    pub usage: TextureUsage,
}

/// A 2D texture array
pub struct TextureArray {
    pub(crate) inner: wgpu::Texture,
    width: u32,
    height: u32,
    array_layers: u32,
    format: TextureFormat,
}

impl GraphResource for TextureArray {}

impl TextureArray {
    pub fn create(device: &Device, info: TextureArrayCreateInfo) -> Self {
        let texture_size = wgpu::Extent3d {
            width: info.width,
            height: info.height,
            depth_or_array_layers: info.array_layers,
        };

        let texture = device.create_texture(&TextureDescriptor {
            label: info.label,
            size: texture_size,
            format: info.format.into(),
            usage: info.usage.into(),
            dimension: TextureDimension::D2,
            mip_level_count: 1,
            sample_count: 1,
            view_formats: &[],
        });

        Self {
            inner: texture,
            width: info.width,
            height: info.height,
            array_layers: info.array_layers,
            format: info.format,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn array_layers(&self) -> u32 {
        self.array_layers
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }

    /// Create a view of the entire array
    pub fn create_view(&self) -> TextureView {
        let view = self.inner.create_view(&TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            array_layer_count: Some(self.array_layers),
            ..Default::default()
        });
        TextureView { inner: view }
    }

    /// Create a view of a single layer for rendering to
    pub fn create_layer_view(&self, layer: u32) -> TextureView {
        let view = self.inner.create_view(&TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2),
            base_array_layer: layer,
            array_layer_count: Some(1),
            ..Default::default()
        });
        TextureView { inner: view }
    }
}

pub struct TextureCubeArrayCreateInfo {
    pub label: Option<&'static str>,
    pub size: u32,
    pub array_layers: u32,
    pub format: TextureFormat,
    pub usage: TextureUsage,
}

/// A cube texture array - useful for point light shadow maps
pub struct TextureCubeArray {
    pub(crate) inner: wgpu::Texture,
    size: u32,
    array_layers: u32,
    format: TextureFormat,
}

impl GraphResource for TextureCubeArray {}

impl TextureCubeArray {
    pub fn create(device: &Device, info: TextureCubeArrayCreateInfo) -> Self {
        let texture_size = wgpu::Extent3d {
            width: info.size,
            height: info.size,
            depth_or_array_layers: info.array_layers * 6,
        };

        let texture = device.create_texture(&TextureDescriptor {
            label: info.label,
            size: texture_size,
            format: info.format.into(),
            usage: info.usage.into(),
            dimension: TextureDimension::D2,
            mip_level_count: 1,
            sample_count: 1,
            view_formats: &[],
        });

        Self {
            inner: texture,
            size: info.size,
            array_layers: info.array_layers,
            format: info.format,
        }
    }

    pub fn size(&self) -> u32 {
        self.size
    }

    pub fn array_layers(&self) -> u32 {
        self.array_layers
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }

    /// Create a view of the entire cube array
    pub fn create_view(&self) -> TextureView {
        let view = self.inner.create_view(&TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::CubeArray),
            array_layer_count: Some(self.array_layers * 6),
            ..Default::default()
        });
        TextureView { inner: view }
    }

    /// Create a view of a single cube face for rendering to
    pub fn create_face_view(&self, cube_index: u32, face: u32) -> TextureView {
        let view = self.inner.create_view(&TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2),
            base_array_layer: cube_index * 6 + face,
            array_layer_count: Some(1),
            ..Default::default()
        });
        TextureView { inner: view }
    }

    /// Create a view of a single cube (all 6 faces)
    pub fn create_cube_view(&self, cube_index: u32) -> TextureView {
        let view = self.inner.create_view(&TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            base_array_layer: cube_index * 6,
            array_layer_count: Some(6),
            ..Default::default()
        });
        TextureView { inner: view }
    }
}
