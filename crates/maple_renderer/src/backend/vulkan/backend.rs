use anyhow::{Context, Result, anyhow};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use vulkano::{
    Validated, VulkanError, VulkanLibrary,
    buffer::{
        Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer,
        allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
    },
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferExecFuture, PrimaryAutoCommandBuffer,
        RenderPassBeginInfo, SubpassBeginInfo, SubpassEndInfo,
        allocator::StandardCommandBufferAllocator,
    },
    descriptor_set::{
        DescriptorSet, WriteDescriptorSet,
        allocator::StandardDescriptorSetAllocator,
        layout::{
            DescriptorSetLayout, DescriptorSetLayoutBinding, DescriptorSetLayoutCreateFlags,
            DescriptorSetLayoutCreateInfo, DescriptorType,
        },
    },
    device::{
        Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures, Queue, QueueCreateInfo,
        QueueFlags,
        physical::{PhysicalDevice, PhysicalDeviceType},
    },
    format::Format,
    image::{Image, ImageLayout, ImageUsage, SampleCount, view::ImageView},
    instance::{Instance, InstanceCreateFlags, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
        graphics::{
            GraphicsPipelineCreateInfo,
            color_blend::{ColorBlendAttachmentState, ColorBlendState},
            input_assembly::InputAssemblyState,
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::{Vertex as VkVertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
    },
    render_pass::{
        AttachmentDescription, AttachmentReference, Framebuffer, FramebufferCreateInfo, RenderPass,
        RenderPassCreateInfo, Subpass, SubpassDescription,
    },
    shader::{ShaderStage, ShaderStages},
    swapchain::{
        self, CompositeAlpha, PresentFuture, Surface, Swapchain, SwapchainAcquireFuture,
        SwapchainCreateInfo, SwapchainPresentInfo,
    },
    sync::{
        self, GpuFuture,
        future::{FenceSignalFuture, JoinFuture},
    },
};

use std::{any::Any, fmt::Debug, sync::Arc};

use crate::{
    backend::vulkan::{
        VulkanBuffer,
        descriptor_set::{VulkanDescriptorSet, VulkanDescriptorSetLayout},
        pipeline::VulkanPipeline,
        render_pass::VulkanRenderPass,
        shader::VulkanShader,
    },
    core::{
        descriptor_set::{
            DescriptorBindingDesc, DescriptorBindingType, DescriptorWrite, StageFlags,
        },
        render_pass::{RenderPassDescriptor, RenderPassWrapper},
        renderer::Renderer,
    },
    types::{Vertex, drawable::Drawable},
};

type FrameFenceFuture = FenceSignalFuture<
    PresentFuture<CommandBufferExecFuture<JoinFuture<Box<dyn GpuFuture>, SwapchainAcquireFuture>>>,
>;

struct FrameState {
    pub image_i: u32,
    pub framebuffer: Arc<Framebuffer>,
    pub acquire_future: SwapchainAcquireFuture,
}

pub struct VulkanBackend {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,

    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    uniform_buffer_allocator: Arc<SubbufferAllocator>,

    swapchain: Arc<Swapchain>,
    framebuffers: Vec<Arc<Framebuffer>>,
    render_pass: Arc<RenderPass>,
    pub viewport: Viewport,

    // option so we can take ownership without re-allocating a future
    previous_frame_end: Option<Box<dyn GpuFuture>>,
    frame_state: Option<FrameState>,

    previous_fence_i: usize,

    recreate_swapchain: bool,
}

impl Debug for VulkanBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VulkanBackend")
            .field("device", &"<Device>")
            .field("queue", &"<Queue>")
            .field("memory_allocator", &"<StandardMemoryAllocator>")
            .field(
                "command_buffer_allocator",
                &"<StandardCommandBufferAllocator>",
            )
            .field(
                "descriptor_set_allocator",
                &"<StandardDescriptorSetAllocator>",
            )
            .field("swapchain", &"<Swapchain>")
            .field("framebuffers", &self.framebuffers.len())
            .field("render_pass", &"<RenderPass>")
            .field("viewport", &self.viewport)
            .field("fences", &"<skipped>")
            .field("previous_fence_i", &self.previous_fence_i)
            .finish()
    }
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

        let supported_features = physical_device.supported_features();

        let features = DeviceFeatures {
            ..Default::default()
        };

        if features != features.intersection(supported_features) {
            eprint!("failed to load all vulkan features");
        }

        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                enabled_extensions: device_extensions,
                enabled_features: features.intersection(supported_features),
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

        let (swapchain, images) = Swapchain::new(
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
        let uniform_buffer_allocator = Arc::new(SubbufferAllocator::new(
            memory_allocator.clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::UNIFORM_BUFFER,
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
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
        let previous_fence_i = 0;

        let previous_frame_end = Some(sync::now(device.clone()).boxed());

        Ok(Self {
            device,
            queue,

            memory_allocator,
            command_buffer_allocator,
            descriptor_set_allocator,
            uniform_buffer_allocator,

            swapchain,
            framebuffers,
            render_pass,
            viewport,

            previous_frame_end,
            frame_state: None,

            previous_fence_i,

            recreate_swapchain: false,
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

    pub fn resize(&mut self, dimensions: [u32; 2]) -> Result<()> {
        let (new_swapchain, new_images) = self
            .swapchain
            .recreate(SwapchainCreateInfo {
                image_extent: dimensions,
                ..self.swapchain.create_info()
            })
            .context("failed to recreate swapchain")?;

        self.swapchain = new_swapchain;

        let new_framebuffers = Self::get_framebuffers(&new_images, &self.render_pass)?;

        self.viewport.extent = dimensions.map(|d| d as f32);

        self.framebuffers = new_framebuffers;
        self.recreate_swapchain = false;

        Ok(())
    }

    pub fn create_buffer_vertex<T, I>(&self, iter: I) -> Result<VulkanBuffer<[T]>>
    where
        T: BufferContents,
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        let buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            iter,
        )
        .context("failed to create vertex buffer")?;

        Ok(VulkanBuffer { buffer })
    }

    pub fn create_buffer_index<T, I>(&self, iter: I) -> Result<VulkanBuffer<[T]>>
    where
        T: BufferContents,
        I: IntoIterator<Item = T>,
        I::IntoIter: ExactSizeIterator,
    {
        let buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::INDEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            iter,
        )
        .context("failed to create index buffer")?;

        Ok(VulkanBuffer { buffer })
    }

    pub fn create_buffer_uniform<T>(&self, data: T) -> Result<VulkanBuffer<T>>
    where
        T: BufferContents,
    {
        let buffer = self.uniform_buffer_allocator.allocate_sized::<T>()?;
        *buffer.write()? = data;
        Ok(VulkanBuffer { buffer })
    }

    pub fn create_descriptor_set_layout(
        &self,
        bindings: &[DescriptorBindingDesc],
    ) -> Result<VulkanDescriptorSetLayout> {
        let vk_bindings: Vec<(u32, DescriptorSetLayoutBinding)> = bindings
            .iter()
            .map(|b| {
                let descriptor_type = match b.bindig_type {
                    DescriptorBindingType::UniformBuffer => DescriptorType::UniformBuffer,
                };

                let layout_binding = DescriptorSetLayoutBinding {
                    stages: b.stages.into(),
                    ..DescriptorSetLayoutBinding::descriptor_type(descriptor_type)
                };
                (b.binding, layout_binding)
            })
            .collect();

        let create_info = DescriptorSetLayoutCreateInfo {
            bindings: vk_bindings.into_iter().collect(), //convert array to BTree
            ..Default::default()
        };

        let layout = DescriptorSetLayout::new(self.device.clone(), create_info)
            .context("fauled to create descriptor set layout")?;
        Ok(VulkanDescriptorSetLayout { layout })
    }

    pub fn create_descriptor_set<T: BufferContents>(
        &self,
        layout: Arc<DescriptorSetLayout>,
        _set_index: u32,
        writes: &[DescriptorWrite<T>],
    ) -> Result<VulkanDescriptorSet> {
        let mut vk_writes: Vec<WriteDescriptorSet> = Vec::with_capacity(writes.len());

        for w in writes {
            match w {
                DescriptorWrite::UniformBuffer { binding, buffer } => {
                    let buffer = (*buffer).clone();
                    let sub = VulkanBuffer::<T>::from(buffer).buffer;
                    vk_writes.push(WriteDescriptorSet::buffer(*binding, sub));
                }
            }
        }

        let set = DescriptorSet::new(self.descriptor_set_allocator.clone(), layout, vk_writes, [])
            .context("failed to build descriptor set")?;

        Ok(VulkanDescriptorSet { set })
    }

    pub fn prepare_pass(
        &mut self,
        pass: &mut RenderPassWrapper,
    ) -> Result<AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>> {
        self.previous_frame_end.as_mut().unwrap().cleanup_finished();

        let (image_i, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(self.swapchain.clone(), None)
                .map_err(Validated::unwrap)
            {
                Ok(r) => r,
                Err(vulkano::VulkanError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return Err(anyhow!("swapchain is out of date"));
                }
                Err(e) => panic!("failed to acquire next image in swapchain: {e}"),
            };

        if suboptimal {
            self.recreate_swapchain = true;
        }

        pass.pass.predraw();

        // create the command buffer
        let mut builder = AutoCommandBufferBuilder::primary(
            self.command_buffer_allocator.clone(),
            self.queue.queue_family_index(),
            vulkano::command_buffer::CommandBufferUsage::OneTimeSubmit,
        )?;

        // bind the target framebuffer and pipeline
        let framebuffer = self.framebuffers[image_i as usize].clone();
        let pipeline: VulkanPipeline = pass.context.pipeline.clone().into();

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                },
                SubpassBeginInfo {
                    contents: vulkano::command_buffer::SubpassContents::Inline,
                    ..Default::default()
                },
            )?
            .bind_pipeline_graphics(pipeline.unbox())?;

        self.frame_state = Some(FrameState {
            image_i,
            framebuffer,
            acquire_future,
        });

        Ok(builder)
    }

    pub fn end_pass(
        &mut self,
        mut builder: AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) -> Result<()> {
        let Some(state) = self.frame_state.take() else {
            return Err(anyhow!("the pass hasnt started"));
        };

        // we are all done we can build and submit
        builder.end_render_pass(SubpassEndInfo::default())?;

        let command_buffer = builder.build()?;

        // submit the frame and create a future for it
        let future = self
            .previous_frame_end
            .take()
            .unwrap()
            .join(state.acquire_future)
            .then_execute(self.queue.clone(), command_buffer)?
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(self.swapchain.clone(), state.image_i),
            )
            .then_signal_fence_and_flush();

        match future.map_err(Validated::unwrap) {
            Ok(future) => {
                self.previous_frame_end = Some(future.boxed());
            }
            Err(VulkanError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_end = Some(sync::now(self.device.clone()).boxed());
            }
            Err(e) => {
                panic!("failed to flush future: {e}");
            }
        }

        Ok(())
    }

    pub fn create_render_pass(&self, info: &RenderPassDescriptor) -> Result<VulkanRenderPass> {
        let mut attachments = vec![AttachmentDescription {
            format: info.format.unwrap_or(Format::default()),
            samples: SampleCount::Sample1,
            load_op: vulkano::render_pass::AttachmentLoadOp::Clear,
            store_op: vulkano::render_pass::AttachmentStoreOp::Store,
            stencil_load_op: Some(vulkano::render_pass::AttachmentLoadOp::DontCare),
            stencil_store_op: Some(vulkano::render_pass::AttachmentStoreOp::DontCare),
            initial_layout: ImageLayout::Undefined,
            final_layout: ImageLayout::PresentSrc,
            ..Default::default()
        }];

        if let Some(depth_format) = info.depth_format {
            attachments.push(AttachmentDescription {
                format: depth_format,
                samples: SampleCount::Sample1,
                load_op: vulkano::render_pass::AttachmentLoadOp::Clear,
                store_op: vulkano::render_pass::AttachmentStoreOp::Store,
                stencil_load_op: Some(vulkano::render_pass::AttachmentLoadOp::DontCare),
                stencil_store_op: Some(vulkano::render_pass::AttachmentStoreOp::DontCare),
                initial_layout: ImageLayout::Undefined,
                final_layout: ImageLayout::PresentSrc,
                ..Default::default()
            });
        }

        let subpasses = vec![SubpassDescription {
            color_attachments: vec![Some(AttachmentReference {
                attachment: 0,
                layout: ImageLayout::ColorAttachmentOptimal,
                ..Default::default()
            })],
            depth_stencil_attachment: if info.depth_format.is_some() {
                Some(AttachmentReference {
                    attachment: 1,
                    layout: ImageLayout::DepthStencilAttachmentOptimal,
                    ..Default::default()
                })
            } else {
                None
            },
            ..Default::default()
        }];

        let pass = RenderPass::new(
            self.device.clone(),
            RenderPassCreateInfo {
                attachments,
                subpasses,
                ..Default::default()
            },
        )?;

        Ok(VulkanRenderPass { render_pass: pass })
    }

    pub fn get_swapchain_pass(&self) -> VulkanRenderPass {
        VulkanRenderPass {
            render_pass: self.render_pass.clone(),
        }
    }

    pub fn get_swapchain_viewport(&self) -> Viewport {
        self.viewport.clone()
    }

    pub fn create_pipeline(
        &self,
        shader: VulkanShader,
        render_pass: VulkanRenderPass,
        viewport: Viewport,
    ) -> Result<VulkanPipeline> {
        println!("compiling shaders...");

        let vs = shader
            .vertex
            .entry_point("main")
            .ok_or_else(|| anyhow!("failed to get vertex entry point"))?;
        let fs = shader
            .fragment
            .entry_point("main")
            .ok_or_else(|| anyhow!("failed to get fragment entry point"))?;

        let vertex_input_state = Vertex::per_vertex().definition(&vs).unwrap();

        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];

        let layout = PipelineLayout::new(
            self.device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(self.device.clone())?,
        )?;

        let subpass = Subpass::from(render_pass.render_pass.clone(), 0).unwrap();

        let pipeline = GraphicsPipeline::new(
            self.device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState {
                    viewports: [viewport].into_iter().collect(),
                    ..Default::default()
                }),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default(),
                )),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )?;

        println!("created pipeline");

        Ok(VulkanPipeline { inner: pipeline })
    }
}

#[cfg(test)]
mod test {
    use crate::types::Vertex;

    use super::*;
    #[test]
    fn test() {}
}
