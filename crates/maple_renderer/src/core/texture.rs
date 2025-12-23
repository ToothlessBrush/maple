use std::sync::Arc;

use bitflags::bitflags;
use parking_lot::RwLock;
use wgpu::{
    AddressMode, Device, Origin3d, Queue, TexelCopyBufferLayout, TexelCopyTextureInfo,
    TextureAspect, TextureDescriptor, TextureDimension, TextureUsages, TextureViewDescriptor,
};

use crate::{
    core::{DepthCompare, RenderContext},
    render_graph::graph::GraphResource,
};

pub struct TextureView {
    pub(crate) inner: wgpu::TextureView,
}

#[derive(Clone)]
pub struct Sampler {
    pub(crate) inner: wgpu::Sampler,
}

impl GraphResource for Sampler {}

pub struct SamplerOptions {
    pub mode_u: TextureMode,
    pub mode_v: TextureMode,
    pub mode_w: TextureMode,
    pub mag_filter: FilterMode,
    pub min_filter: FilterMode,
    pub compare: Option<DepthCompare>,
}

impl From<SamplerOptions> for wgpu::SamplerDescriptor<'static> {
    fn from(value: SamplerOptions) -> Self {
        Self {
            address_mode_u: value.mode_u.into(),
            address_mode_v: value.mode_v.into(),
            address_mode_w: value.mode_w.into(),
            mag_filter: value.mag_filter.into(),
            min_filter: value.min_filter.into(),
            compare: value.compare.map(|c| c.into()),
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
    #[derive(Clone, Copy)]
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

#[derive(PartialEq, Eq, Clone)]
pub struct Texture {
    pub(crate) inner: wgpu::Texture,
    width: u32,
    height: u32,
    format: TextureFormat,
    /// Optional array layer to use when creating views (for rendering to specific layers)
    array_layer: Option<u32>,
}

impl GraphResource for Texture {}

impl Texture {
    pub(crate) fn create(device: &Device, info: &TextureCreateInfo) -> Self {
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
            width: info.width,
            format: info.format,
            array_layer: None,
        }
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub(crate) fn write(&self, queue: &Queue, data: &[u8]) {
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
        let view = if let Some(layer) = self.array_layer {
            // Create view for specific array layer
            self.inner.create_view(&TextureViewDescriptor {
                dimension: Some(wgpu::TextureViewDimension::D2),
                base_array_layer: layer,
                array_layer_count: Some(1),
                ..Default::default()
            })
        } else {
            // Create default view
            self.inner.create_view(&TextureViewDescriptor::default())
        };
        TextureView { inner: view }
    }

    pub(crate) fn create_sampler(device: &Device, options: SamplerOptions) -> Sampler {
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
#[derive(Clone)]
pub struct TextureArray {
    pub(crate) inner: wgpu::Texture,
    width: u32,
    height: u32,
    array_layers: u32,
    format: TextureFormat,
}

impl GraphResource for TextureArray {}

impl TextureArray {
    pub(crate) fn create(device: &Device, info: &TextureArrayCreateInfo) -> Self {
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

    pub fn lazy(data: Vec<u8>, info: TextureCreateInfo) -> LazyTexture {
        LazyTexture::new(data, info)
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

    /// Create a Texture wrapper for a specific layer (for use as render target)
    /// This shares the underlying wgpu::Texture but creates views for the specific layer
    pub fn create_layer_texture(&self, layer: u32) -> Texture {
        Texture {
            inner: self.inner.clone(),
            width: self.width,
            height: self.height,
            format: self.format,
            array_layer: Some(layer),
        }
    }
}

#[derive(Clone, Copy)]
pub struct TextureCubeArrayCreateInfo {
    pub label: Option<&'static str>,
    pub size: u32,
    pub array_layers: u32,
    pub format: TextureFormat,
    pub usage: TextureUsage,
}

/// A cube texture array - useful for point light shadow maps
#[derive(Clone)]
pub struct TextureCubeArray {
    pub(crate) inner: wgpu::Texture,
    size: u32,
    array_layers: u32,
    format: TextureFormat,
}

impl GraphResource for TextureCubeArray {}

impl TextureCubeArray {
    pub(crate) fn create(device: &Device, info: &TextureCubeArrayCreateInfo) -> Self {
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

    /// Create a Texture wrapper for a specific cube face (for use as render target)
    pub fn create_face_texture(&self, cube_index: u32, face: u32) -> Texture {
        Texture {
            inner: self.inner.clone(),
            width: self.size,
            height: self.size,
            format: self.format,
            array_layer: Some(cube_index * 6 + face),
        }
    }
}

enum LazyTextureState {
    Pending(Vec<u8>, TextureCreateInfo),
    Clean(Texture),
}

#[derive(Clone)]
pub struct LazyTexture {
    state: Arc<RwLock<LazyTextureState>>,
}

impl LazyTexture {
    pub fn new(data: Vec<u8>, info: TextureCreateInfo) -> Self {
        Self {
            state: Arc::new(RwLock::new(LazyTextureState::Pending(data, info))),
        }
    }

    pub fn texture(&self, rcx: &RenderContext) -> Texture {
        rcx.get_texture(self)
    }

    pub(crate) fn get_texture(&self, device: &Device, queue: &Queue) -> Texture {
        {
            let read_guard = self.state.read();
            if let LazyTextureState::Clean(texture) = &*read_guard {
                return texture.clone();
            }
        }

        let mut write_guard = self.state.write();
        match &*write_guard {
            LazyTextureState::Pending(data, info) => {
                let texture = Texture::create(device, info);
                texture.write(queue, data);
                let result = texture.clone();
                *write_guard = LazyTextureState::Clean(texture);
                result
            }
            LazyTextureState::Clean(texture) => texture.clone(),
        }
    }
}

impl From<Texture> for LazyTexture {
    fn from(value: Texture) -> Self {
        LazyTexture {
            state: Arc::new(RwLock::new(LazyTextureState::Clean(value))),
        }
    }
}
