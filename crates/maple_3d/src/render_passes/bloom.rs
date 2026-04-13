use std::slice;

use bytemuck::{Pod, Zeroable};
use maple_renderer::{
    core::{
        AlphaMode, Buffer, ComputePipeline, ComputePipelineCreateInfo, CullMode,
        DescriptorBindingType, DescriptorSet, DescriptorSetLayout, PipelineCreateInfo,
        RenderContext, RenderPipeline, StageFlags,
        context::{Dimensions, RenderOptions},
        texture::{
            FilterMode, Sampler, SamplerOptions, Texture, TextureCreateInfo, TextureFormat,
            TextureMode, TextureUsage,
        },
    },
    render_graph::{
        graph::RenderGraphContext,
        node::{DepthMode, RenderNode, RenderTarget},
    },
};

const MIP_LEVELS: u32 = 5;
const WORKGROUP_SIZE: u32 = 8;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct DownsampleUniforms {
    src_resolution: [f32; 2],
    _padding: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct UpsampleUniforms {
    filter_radius: f32,
    _padding: [f32; 3],
}

pub struct BloomPass {
    downsample_pipeline: ComputePipeline,
    downsample_layout: DescriptorSetLayout,

    upsample_pipeline: RenderPipeline,
    upsample_layout: DescriptorSetLayout,

    bright_pipeline: ComputePipeline,
    bright_layout: DescriptorSetLayout,

    sampler: Sampler,

    mip_chain: Vec<Texture>,
    downsample_uniforms: Vec<Buffer<DownsampleUniforms>>,
    upsample_uniform: Buffer<UpsampleUniforms>,
}

impl BloomPass {
    pub fn setup(rcx: &RenderContext, _: &mut RenderGraphContext) -> Self {
        let bright_shader =
            rcx.create_compute_shader(maple_renderer::core::ComputeShaderSource::Wgsl(
                include_str!("../../res/shaders/bloom/bright.wgsl"),
            ));

        let downsample_shader =
            rcx.create_compute_shader(maple_renderer::core::ComputeShaderSource::Wgsl(
                include_str!("../../res/shaders/bloom/downsample.wgsl"),
            ));

        let upsample_shader = rcx.create_shader_pair(maple_renderer::core::ShaderPair::Wgsl {
            vert: include_str!("../../res/shaders/bloom/upsample.vert.wgsl"),
            frag: include_str!("../../res/shaders/bloom/upsample.frag.wgsl"),
        });

        let downsample_layout =
            rcx.create_descriptor_set_layout(maple_renderer::core::DescriptorSetLayoutDescriptor {
                label: Some("bloom_downsample_layout"),
                visibility: StageFlags::COMPUTE,
                layout: &[
                    DescriptorBindingType::TextureView { filterable: true },
                    DescriptorBindingType::Sampler { filtering: true },
                    DescriptorBindingType::StorageTexture2D {
                        format: TextureFormat::RGBA16Float,
                        access: maple_renderer::core::StorageAccess::WriteOnly,
                    },
                    DescriptorBindingType::UniformBuffer,
                ],
            });

        let upsample_layout =
            rcx.create_descriptor_set_layout(maple_renderer::core::DescriptorSetLayoutDescriptor {
                label: Some("bloom_upsample_layout"),
                visibility: StageFlags::FRAGMENT,
                layout: &[
                    DescriptorBindingType::TextureView { filterable: true },
                    DescriptorBindingType::Sampler { filtering: true },
                    DescriptorBindingType::UniformBuffer,
                ],
            });

        let bright_layout =
            rcx.create_descriptor_set_layout(maple_renderer::core::DescriptorSetLayoutDescriptor {
                label: Some("bloom_bright_layout"),
                visibility: StageFlags::COMPUTE,
                layout: &[
                    DescriptorBindingType::TextureView { filterable: true },
                    DescriptorBindingType::StorageTexture2D {
                        format: TextureFormat::RGBA16Float,
                        access: maple_renderer::core::StorageAccess::WriteOnly,
                    },
                ],
            });

        let downsample_pipeline_layout =
            rcx.create_pipeline_layout(slice::from_ref(&downsample_layout));
        let upsample_pipeline_layout =
            rcx.create_pipeline_layout(slice::from_ref(&upsample_layout));
        let bright_pipeline_layout = rcx.create_pipeline_layout(slice::from_ref(&bright_layout));

        let downsample_pipeline = rcx.create_compute_pipeline(ComputePipelineCreateInfo {
            label: Some("bloom_downscale"),
            layout: downsample_pipeline_layout,
            shader: downsample_shader,
            entry_point: None,
        });

        let upsample_pipeline = rcx.create_pipeline(PipelineCreateInfo {
            label: Some("bloom_upsample"),
            layout: upsample_pipeline_layout,
            shader: upsample_shader,
            color_formats: &[TextureFormat::RGBA16Float],
            depth: &DepthMode::None,
            cull_mode: CullMode::None,
            alpha_mode: AlphaMode::Additive, // src + dst blending
            sample_count: 1,
            use_vertex_buffer: false,
        });

        let bright_pipeline = rcx.create_compute_pipeline(ComputePipelineCreateInfo {
            label: Some("bright"),
            layout: bright_pipeline_layout,
            shader: bright_shader,
            entry_point: None,
        });

        let sampler = rcx.create_sampler(SamplerOptions {
            mode_u: TextureMode::ClampToEdge,
            mode_v: TextureMode::ClampToEdge,
            mode_w: TextureMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            compare: None,
        });

        let upsample_uniform = rcx.create_uniform_buffer(&UpsampleUniforms {
            filter_radius: 0.005,
            _padding: [0.0; 3],
        });

        Self {
            downsample_pipeline,
            downsample_layout,
            upsample_pipeline,
            upsample_layout,
            bright_pipeline,
            bright_layout,
            sampler,
            mip_chain: Vec::new(),
            downsample_uniforms: Vec::new(),
            upsample_uniform,
        }
    }

    fn create_mip_chain(&mut self, rcx: &RenderContext, width: u32, height: u32) {
        self.mip_chain.clear();
        self.downsample_uniforms.clear();

        let mut w = width;
        let mut h = height;

        for _ in 0..MIP_LEVELS {
            let texture = rcx.create_texture(TextureCreateInfo {
                label: Some("Bloom_mip"),
                width: w,
                height: h,
                format: TextureFormat::RGBA16Float,
                usage: TextureUsage::TEXTURE_BINDING
                    | TextureUsage::STORAGE_BINDING
                    | TextureUsage::RENDER_ATTACHMENT,
                mip_level: 1,
                sample_count: 1,
            });
            self.mip_chain.push(texture);

            let uniform = rcx.create_uniform_buffer(&DownsampleUniforms {
                src_resolution: [w as f32 * 2.0, h as f32 * 2.0],
                _padding: [0.0; 2],
            });
            self.downsample_uniforms.push(uniform);

            w = (w / 2).max(1);
            h = (h / 2).max(1);
        }
    }
}

impl RenderNode for BloomPass {
    fn draw(
        &mut self,
        rcx: &RenderContext,
        graph_ctx: &mut RenderGraphContext,
        _: &maple_engine::GameContext,
    ) {
        let Some(resolved_texture) =
            graph_ctx.get_shared_resource::<Texture>("resolved_color_texture")
        else {
            return;
        };

        if self.mip_chain.len() <= 2 {
            self.create_mip_chain(rcx, rcx.surface_size().width, rcx.surface_size().height);
        }

        // bright pass
        {
            let descriptor = rcx.build_descriptor_set(
                DescriptorSet::builder(&self.bright_layout)
                    .texture_view(0, &resolved_texture.create_view())
                    .texture_view(1, &self.mip_chain[0].create_view()),
            );

            let dispatch_x = self.mip_chain[0].width().div_ceil(WORKGROUP_SIZE);
            let dispatch_y = self.mip_chain[0].height().div_ceil(WORKGROUP_SIZE);

            rcx.compute(Some("bloom_downsample"), |mut cb| {
                cb.use_pipeline(&self.bright_pipeline)
                    .bind_descriptor_set(0, &descriptor)
                    .dispatch(dispatch_x, dispatch_y, 1);
            });
        }

        // downsample for the mip chain
        for i in 1..self.mip_chain.len() {
            let descriptor = rcx.build_descriptor_set(
                DescriptorSet::builder(&self.downsample_layout)
                    .texture_view(0, &self.mip_chain[i - 1].create_view())
                    .sampler(1, &self.sampler)
                    .texture_view(2, &self.mip_chain[i].create_view())
                    .uniform(3, &self.downsample_uniforms[i]),
            );

            let dispatch_x = self.mip_chain[i].width().div_ceil(WORKGROUP_SIZE);
            let dispatch_y = self.mip_chain[i].height().div_ceil(WORKGROUP_SIZE);

            rcx.compute(Some("bloom_downsample"), |mut cb| {
                cb.use_pipeline(&self.downsample_pipeline)
                    .bind_descriptor_set(0, &descriptor)
                    .dispatch(dispatch_x, dispatch_y, 1);
            });
        }

        // upsample through the mip chain
        for i in (0..self.mip_chain.len() - 1).rev() {
            let desc = rcx.build_descriptor_set(
                DescriptorSet::builder(&self.upsample_layout)
                    .texture_view(0, &self.mip_chain[i + 1].create_view()) // src: smaller mip
                    .sampler(1, &self.sampler)
                    .uniform(2, &self.upsample_uniform), // binding 2 now (no storage texture)
            );

            let clear_color = if i != 0 {
                None
            } else {
                Some([0.0, 0.0, 0.0, 1.0])
            };

            rcx.render(
                RenderOptions {
                    label: Some("bloom_upsample"),
                    color_targets: &[RenderTarget::Texture(self.mip_chain[i].create_view())], // dst: larger mip
                    depth_target: None,
                    clear_color, // DON'T clear - additive blend onto existing downsample data
                    clear_depth: None,
                },
                |mut fb| {
                    fb.use_pipeline(&self.upsample_pipeline)
                        .bind_descriptor_set(0, &desc)
                        .draw(0..3); // fullscreen triangle                                                                                                                                                                                                                            
                },
            )
            .expect("bloom upsample failed");
        }

        graph_ctx.add_shared_resource("bloom_texture", self.mip_chain[0].clone());
    }

    fn resize(&mut self, rcx: &RenderContext, dimensions: Dimensions) {
        self.create_mip_chain(rcx, dimensions.width, dimensions.height);
    }
}
