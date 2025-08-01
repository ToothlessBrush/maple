use anyhow::{Context, Result, anyhow};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use vulkano::{
    VulkanLibrary,
    command_buffer::{CommandBufferExecFuture, allocator::StandardCommandBufferAllocator},
    descriptor_set::allocator::StandardDescriptorSetAllocator,
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags,
        physical::{PhysicalDevice, PhysicalDeviceType},
    },
    image::{Image, ImageUsage, view::ImageView},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::StandardMemoryAllocator,
    pipeline::graphics::viewport::Viewport,
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
    swapchain::{
        CompositeAlpha, PresentFuture, Surface, Swapchain, SwapchainAcquireFuture,
        SwapchainCreateInfo,
    },
    sync::{
        GpuFuture,
        future::{FenceSignalFuture, JoinFuture},
    },
};

use std::{any::Any, sync::Arc};

type FrameFenceFuture = FenceSignalFuture<
    PresentFuture<CommandBufferExecFuture<JoinFuture<Box<dyn GpuFuture>, SwapchainAcquireFuture>>>,
>;

pub struct VulkanBackend {
    device: Arc<Device>,
    queue: Arc<Queue>,

    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,

    swapchain: Arc<Swapchain>,
    framebuffers: Vec<Arc<Framebuffer>>,
    render_pass: Arc<RenderPass>,
    viewport: Viewport,

    fences: Vec<Option<Arc<FrameFenceFuture>>>,
    previous_fence_i: usize,
}

impl VulkanBackend {
    pub fn init(
        window: Arc<impl HasDisplayHandle + HasWindowHandle + Any + Send + Sync>,
        dimensions: [u32; 2],
    ) -> Result<Self> {
        let required_extensions = Surface::required_extensions(&*window)?;

        let library = VulkanLibrary::new().context("no vulkan library/dll")?;
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                enabled_extensions: required_extensions,
                ..Default::default()
            },
        )
        .context("failed to create vulkan instance")?;

        let surface = Surface::from_window(instance.clone(), window.clone())?;

        // device creation

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) =
            Self::select_physical_device(&instance, &surface, &device_extensions)
                .context("could select physical device")?;

        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                enabled_extensions: device_extensions,
                ..Default::default()
            },
        )
        .context("failed to create device")?;

        let queue = queues
            .next()
            .ok_or_else(|| anyhow!("failed to grab queue"))?;

        // swapchain creation

        let caps = physical_device
            .surface_capabilities(&surface, Default::default())
            .context("failed to grab surface capabilities")?;

        let composite_alpha = caps
            .supported_composite_alpha
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("composite alpha"))?;

        let image_format = physical_device.surface_formats(&surface, Default::default())?[0].0;

        let (mut swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: caps.min_image_count + 1,
                image_format,
                image_extent: dimensions,
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha,
                ..Default::default()
            },
        )?;

        // memory allocators

        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));
        let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        // viewport

        let image_extent = swapchain.image_extent().map(|d| d as f32);

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: image_extent,
            depth_range: 0.0..=1.0,
        };

        let render_pass = vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                format: swapchain.image_format(),
                samples: 1,
                load_op: Clear,
                store_op: Store,
            },
        },
        pass: {
            color: [color],
            depth_stencil: {},
        },
        )
        .context("failed to create renderpass")?;

        let framebuffers = Self::get_framebuffers(&images, &render_pass)?;

        // fences

        let frames_in_flight = images.len();
        let fences: Vec<Option<Arc<FenceSignalFuture<_>>>> = vec![None; frames_in_flight];
        let previous_fence_i = 0;

        Ok(Self {
            device,
            queue,

            memory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,

            swapchain,
            framebuffers,
            render_pass,
            viewport,

            fences,
            previous_fence_i,
        })
    }

    /// returns the optimal device or an error if it cant find one
    fn select_physical_device(
        instance: &Arc<Instance>,
        surface: &Arc<Surface>,
        device_extensions: &DeviceExtensions,
    ) -> Result<(Arc<PhysicalDevice>, u32)> {
        instance
            .enumerate_physical_devices()
            .context("could not enumerate devices")?
            .filter(|p| p.supported_extensions().contains(device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.contains(QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, surface).unwrap_or(false)
                    })
                    .map(|q| (p, q as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 4,
                _ => 4,
            })
            .context("no devices avaliable")
    }

    fn get_framebuffers(
        images: &[Arc<Image>],
        render_pass: &Arc<RenderPass>,
    ) -> Result<Vec<Arc<Framebuffer>>> {
        images
            .iter()
            .map(|image| {
                let view = ImageView::new_default(image.clone())
                    .context("failed to create image view for image")?;
                Framebuffer::new(
                    render_pass.clone(),
                    FramebufferCreateInfo {
                        attachments: vec![view],
                        ..Default::default()
                    },
                )
                .context("failed to create framebuffer")
            })
            .collect::<Result<Vec<_>, _>>()
    }
}
