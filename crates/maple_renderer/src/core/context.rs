use super::texture;
use crate::core::{Frame, RenderDevice, RenderQueue, RenderTarget};
use crate::platform::SendSync;
use crate::types::Dimensions;
use crate::{
    core::{
        descriptor_set::{DescriptorSetLayout, DescriptorSetLayoutDescriptor},
        mipmap_generator::{self, MipmapGenerator},
        texture::{Texture, TextureCube, TextureView},
    },
    types::{
        default_texture::DefaultTexture,
        render_config::{RenderConfig, VsyncMode},
    },
};
use parking_lot::RwLock;
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::collections::HashMap;
use std::fmt::Display;
use std::{
    error::Error,
    sync::{Arc, OnceLock},
};
use wgpu::{
    Adapter, Device, DeviceDescriptor, Instance, InstanceDescriptor, PresentMode, Queue,
    RequestAdapterError, RequestAdapterOptions, RequestDeviceError, Surface, SurfaceConfiguration,
    SurfaceTexture, TextureFormat, TextureUsages,
};

pub use wgpu::CreateSurfaceError;

pub struct RenderOptions<'a> {
    pub label: Option<&'a str>,
    pub color_targets: &'a [RenderTarget],
    pub depth_target: Option<&'a TextureView>,
    pub clear_color: Option<[f32; 4]>,
    pub clear_depth: Option<f32>,
}

#[derive(Debug)]
pub enum SurfaceError {
    /// if the surface was attempted to be acquired but no surface has been attached yet
    SurfaceMissing,
    Timeout,
    Occluded,
    Validation,
    Outdated,
    ContextLost,
}

impl Display for SurfaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SurfaceError::SurfaceMissing => write!(f, "surface missing"),
            SurfaceError::Timeout => write!(f, "surface timeout"),
            SurfaceError::Occluded => write!(f, "surface occluded"),
            SurfaceError::Outdated => write!(f, "surface outdated"),
            SurfaceError::Validation => write!(f, "surface validation error occured"),
            SurfaceError::ContextLost => write!(f, "context lost"),
        }
    }
}

impl Error for SurfaceError {}

#[derive(Debug)]
pub enum InitError {
    AdapterError(RequestAdapterError),
    DeviceError(RequestDeviceError),
    SurfaceError(CreateSurfaceError),
}

impl Display for InitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InitError::AdapterError(err) => write!(f, "requesting adapter failed: {err}"),
            InitError::DeviceError(err) => write!(f, "requesting device failed: {err}"),
            InitError::SurfaceError(err) => write!(f, "creating surface failed: {err}"),
        }
    }
}

impl Error for InitError {}

/// holds all raw WGPU state
struct Backend {
    instance: Instance,
    adapter: Adapter,
    device: Arc<Device>,
    queue: Arc<Queue>,
    surface: Option<Surface<'static>>,
    surface_format: texture::TextureFormat,
    config: RenderConfig,
    dimensions: Dimensions,

    default_textures: OnceLock<DefaultTexture>,
    mipmap_generator: MipmapGenerator,
}

impl Backend {
    async fn init<T>(window: Arc<T>, config: RenderConfig) -> Result<Self, InitError>
    where
        T: HasDisplayHandle + HasWindowHandle + SendSync + 'static,
    {
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle());

        let adapter = instance
            .request_adapter(&RequestAdapterOptions::default())
            .await
            .map_err(|err| InitError::AdapterError(err))?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                ..Default::default()
            })
            .await
            .map_err(|err| InitError::DeviceError(err))?;

        let surface: Surface = instance
            .create_surface(window)
            .map_err(|err| InitError::SurfaceError(err))?;

        let cap = surface.get_capabilities(&adapter);
        let surface_format: texture::TextureFormat = cap.formats[0].into();
        println!("SURFACE formats: {:?}", cap.formats);

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let mipmap_generator = MipmapGenerator::new(device.clone(), queue.clone());

        let backend = Self {
            instance: instance,
            adapter,
            device: device,
            queue: queue,
            surface: Some(surface),
            surface_format,
            config,
            dimensions: Dimensions::zero(),
            default_textures: OnceLock::new(),
            mipmap_generator,
        };

        backend.configure_surface();

        Ok(backend)
    }

    async fn init_headless(config: RenderConfig) -> Result<Self, InitError> {
        let instance = Instance::new(InstanceDescriptor::new_without_display_handle());

        let adapter = instance
            .request_adapter(&RequestAdapterOptions::default())
            .await
            .map_err(|err| InitError::AdapterError(err))?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                ..Default::default()
            })
            .await
            .map_err(|err| InitError::DeviceError(err))?;

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let mipmap_generator = MipmapGenerator::new(device.clone(), queue.clone());

        let backend = Self {
            instance: instance,
            adapter,
            device: device,
            queue: queue,
            surface: None,
            surface_format: texture::TextureFormat::BGRA8Srgb,
            config,
            dimensions: Dimensions::zero(),
            default_textures: OnceLock::new(),
            mipmap_generator,
        };

        Ok(backend)
    }

    fn attach_surface<T>(
        &mut self,
        window: Arc<T>,
        dimensions: Dimensions,
    ) -> Result<(), CreateSurfaceError>
    where
        T: HasDisplayHandle + HasWindowHandle + SendSync + 'static,
    {
        let surface: Surface = self.instance.create_surface(window)?;
        let cap = surface.get_capabilities(&self.adapter);
        self.surface_format = cap.formats[0].into();
        self.surface = Some(surface);
        self.dimensions = dimensions;
        self.configure_surface();
        Ok(())
    }

    fn configure_surface(&self) {
        if self.dimensions.width == 0 || self.dimensions.height == 0 {
            return; // nothing to configure yet — wait for a real Resized event
        }

        let Some(surface) = self.surface.as_ref() else {
            return;
        };
        let format: TextureFormat = self.surface_format.into();
        surface.configure(
            &self.device,
            &SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                color_space: wgpu::SurfaceColorSpace::Auto,
                format,
                view_formats: vec![format.add_srgb_suffix()],
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                width: self.dimensions.width,
                height: self.dimensions.height,
                desired_maximum_frame_latency: 3,
                present_mode: match self.config.vsync {
                    VsyncMode::Off => PresentMode::AutoNoVsync,
                    VsyncMode::On => PresentMode::AutoVsync,
                },
            },
        );
    }

    pub fn acquire_surface_texture(&mut self) -> Result<SurfaceTexture, SurfaceError> {
        let surface = self.surface.as_ref().ok_or(SurfaceError::SurfaceMissing)?;
        match surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => Ok(surface_texture),
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => {
                self.configure_surface();
                Ok(surface_texture)
            }
            wgpu::CurrentSurfaceTexture::Timeout => Err(SurfaceError::Timeout),
            wgpu::CurrentSurfaceTexture::Occluded => Err(SurfaceError::Occluded),
            wgpu::CurrentSurfaceTexture::Validation => Err(SurfaceError::Validation),
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.configure_surface();
                Err(SurfaceError::Outdated)
            }
            wgpu::CurrentSurfaceTexture::Lost => Err(SurfaceError::ContextLost),
        }
    }

    pub fn present_surface(&mut self, surface_texture: SurfaceTexture) {
        self.queue.present(surface_texture);
    }

    pub fn resize(&mut self, new_size: Dimensions) {
        self.dimensions = new_size;

        self.configure_surface();
    }

    pub fn change_vsync(&mut self, mode: VsyncMode) {
        self.config.vsync = mode;

        self.configure_surface();
    }
}

/// Public rendering context that provides a safe API over the backend
pub struct RenderContext {
    backend: Backend,
    layout_cache: RwLock<HashMap<DescriptorSetLayoutDescriptor, DescriptorSetLayout>>,
    device: RenderDevice,
    queue: RenderQueue,
}

impl RenderContext {
    pub async fn init<T>(window: Arc<T>, config: RenderConfig) -> Result<Self, InitError>
    where
        T: HasDisplayHandle + HasWindowHandle + SendSync + 'static,
    {
        let backend = Backend::init(window, config).await?;
        Ok(Self {
            layout_cache: RwLock::new(HashMap::new()),
            device: RenderDevice {
                device: backend.device.clone(),
                queue: backend.queue.clone(),
            },
            queue: RenderQueue {
                queue: backend.queue.clone(),
            },
            backend,
        })
    }

    pub async fn init_headless(config: RenderConfig) -> Result<Self, InitError> {
        let backend = Backend::init_headless(config).await?;
        Ok(Self {
            layout_cache: RwLock::new(HashMap::new()),
            device: RenderDevice {
                device: backend.device.clone(),
                queue: backend.queue.clone(),
            },
            queue: RenderQueue {
                queue: backend.queue.clone(),
            },
            backend,
        })
    }

    pub fn create_frame(&self, surface_texture: SurfaceTexture) -> Frame {
        let encoder = self
            .device
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame command encoder"),
            });

        Frame {
            encoder: encoder,
            frame_surface_texture: surface_texture,
        }
    }

    pub fn submit_frame(&self, frame: Frame) -> SurfaceTexture {
        let Frame {
            encoder,
            frame_surface_texture,
            ..
        } = frame;
        self.queue.queue.submit(std::iter::once(encoder.finish()));
        frame_surface_texture
    }

    pub fn attach_surface<T>(
        &mut self,
        window: Arc<T>,
        dimensions: Dimensions,
    ) -> Result<(), CreateSurfaceError>
    where
        T: HasDisplayHandle + HasWindowHandle + SendSync + 'static,
    {
        self.backend.attach_surface(window, dimensions)
    }

    // pub fn get_surface_texture(&self) -> Option<&SurfaceTexture> {
    //     self.backend.get_surface_texture()
    // }

    pub fn get_or_create_layout(
        &self,
        descriptor: DescriptorSetLayoutDescriptor,
    ) -> DescriptorSetLayout {
        {
            let cache = self.layout_cache.read();
            if let Some(layout) = cache.get(&descriptor) {
                return layout.clone();
            }
        }

        let layout = self.device.create_descriptor_set_layout(descriptor);
        self.layout_cache.write().insert(descriptor, layout.clone());
        layout
    }

    /// the rendering device for creating rendering resources
    pub fn device(&self) -> &RenderDevice {
        &self.device
    }

    /// the render queue for queueing render operations
    pub fn queue(&self) -> &RenderQueue {
        &self.queue
    }

    /// generate mipmaps for [`Texture`]
    pub fn mipmap_generator(&self) -> &MipmapGenerator {
        &self.backend.mipmap_generator
    }

    pub fn surface_format(&self) -> texture::TextureFormat {
        self.backend.surface_format
    }

    pub fn resize(&mut self, new_size: Dimensions) {
        self.backend.resize(new_size);
    }

    pub fn change_vsync(&mut self, mode: VsyncMode) {
        self.backend.change_vsync(mode);
    }

    pub fn acquire_surface_texture(&mut self) -> Result<SurfaceTexture, SurfaceError> {
        self.backend.acquire_surface_texture()
    }

    pub fn present_surface(&mut self, surface_texture: SurfaceTexture) {
        self.backend.present_surface(surface_texture)
    }

    pub fn surface_size(&self) -> Dimensions {
        self.backend.dimensions
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.backend.dimensions.width as f32 / self.backend.dimensions.height.max(1) as f32
    }

    pub fn get_default_texture(&self) -> &DefaultTexture {
        self.backend.default_textures.get_or_init(|| {
            DefaultTexture::init_textures(&self.backend.device, &self.backend.queue)
        })
    }

    pub fn generate_mipmaps(&self, texture: &Texture, mip_level_count: u32) {
        mipmap_generator::generate_mipmaps(
            &self.backend.mipmap_generator,
            &self.backend.device,
            &self.backend.queue,
            &texture.inner,
            mip_level_count,
        );
    }

    pub fn generate_cubemap_mipmaps(&self, cubemap: &TextureCube, mip_level_count: u32) {
        mipmap_generator::generate_cubemap_mipmaps(
            &self.backend.mipmap_generator,
            &self.backend.device,
            &self.backend.queue,
            &cubemap.inner,
            mip_level_count,
        );
    }
}
