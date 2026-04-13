use std::slice;

use bytemuck::{Pod, Zeroable};
use maple_engine::GameContext;
use maple_renderer::{
    core::{
        Buffer, CullMode, DescriptorBindingType, DescriptorSet, DescriptorSetLayout,
        DescriptorSetLayoutDescriptor, RenderContext, StageFlags,
        context::{Dimensions, RenderOptions},
        pipeline::{AlphaMode, PipelineCreateInfo, RenderPipeline},
        texture::{FilterMode, Sampler, SamplerOptions, Texture, TextureMode},
    },
    render_graph::{
        graph::RenderGraphContext,
        node::{DepthMode, RenderNode, RenderTarget},
    },
};

use crate::prelude::Camera3D;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CompositeUniforms {
    bloom_intensity: f32,
    exposure: f32,
    _padding: [f32; 2],
}

/// Post-processing pass that blits the resolved color texture to the surface
///
/// This pass:
/// - Reads the resolved color texture (after MSAA resolve)
/// - Renders a fullscreen triangle
/// - Outputs to the surface
///
/// Future post-processing effects (tone mapping, bloom, etc.) can be added here
pub struct CompositePass {
    blit_layout: DescriptorSetLayout,
    blit_descriptor: Option<DescriptorSet>,
    sampler: Sampler,
    pipeline: RenderPipeline,
    uniform: Buffer<CompositeUniforms>,
}

impl CompositePass {
    pub fn setup(rcx: &RenderContext, _gcx: &mut RenderGraphContext) -> Self {
        // Load fullscreen triangle shaders
        let shader = rcx.create_shader_pair(maple_renderer::core::ShaderPair::Wgsl {
            vert: include_str!("../../res/shaders/post_process/blit.vert.wgsl"),
            frag: include_str!("../../res/shaders/post_process/blit.frag.wgsl"),
        });

        // Create descriptor layout for texture + sampler binding
        let blit_layout = rcx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
            label: Some("post_process_blit_layout"),
            visibility: StageFlags::FRAGMENT,
            layout: &[
                DescriptorBindingType::TextureView { filterable: true }, // Binding 0: resolved color texture
                DescriptorBindingType::TextureView { filterable: true }, // Binding 1: Bloom
                DescriptorBindingType::Sampler { filtering: true }, // Binding 2: linear sampler
                DescriptorBindingType::UniformBuffer,
            ],
        });

        // Create sampler once (never changes)
        let sampler = rcx.create_sampler(SamplerOptions {
            mode_u: TextureMode::ClampToEdge,
            mode_v: TextureMode::ClampToEdge,
            mode_w: TextureMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            compare: None,
        });

        // Create pipeline
        let pipeline_layout = rcx.create_pipeline_layout(slice::from_ref(&blit_layout));

        let depth_mode = DepthMode::None;

        let surface_format = rcx.surface_format();

        let pipeline = rcx.create_pipeline(PipelineCreateInfo {
            label: Some("PostProcessPass"),
            layout: pipeline_layout,
            shader: shader.clone(),
            color_formats: &[surface_format],
            depth: &depth_mode,
            cull_mode: CullMode::None,
            alpha_mode: AlphaMode::Opaque,
            sample_count: 1,
            use_vertex_buffer: false,
        });
        let uniform = rcx.create_uniform_buffer(&CompositeUniforms {
            bloom_intensity: 0.04,
            exposure: 0.5,
            _padding: [0.0; 2],
        });

        Self {
            blit_layout,
            blit_descriptor: None,
            sampler,
            pipeline,
            uniform,
        }
    }
}

impl RenderNode for CompositePass {
    fn draw(
        &mut self,
        rcx: &RenderContext,
        graph_ctx: &mut RenderGraphContext,
        game_ctx: &GameContext,
    ) {
        // Get the resolved color texture from graph context
        let Some(resolved_texture) = graph_ctx
            .get_shared_resource::<maple_renderer::core::texture::Texture>(
                "resolved_color_texture",
            )
        else {
            return;
        };

        let cameras = game_ctx.scene.collect::<Camera3D>();
        let Some(camera) = cameras
            .iter()
            .filter(|c| c.read().is_active)
            .max_by_key(|c| c.read().priority)
        else {
            return;
        };

        let exposure = camera.read().exposure;

        rcx.write_buffer(
            &self.uniform,
            &CompositeUniforms {
                bloom_intensity: 0.04,
                exposure,
                _padding: [0.0; 2],
            },
        );

        let bloom_texture = graph_ctx
            .get_shared_resource::<Texture>("bloom_texture")
            .unwrap();

        // Build descriptor once (invalidated on resize)
        if self.blit_descriptor.is_none() {
            let layout = &self.blit_layout;

            self.blit_descriptor = Some(
                rcx.build_descriptor_set(
                    DescriptorSet::builder(layout)
                        .texture_view(0, &resolved_texture.create_view())
                        .texture_view(1, &bloom_texture.create_view())
                        .sampler(2, &self.sampler)
                        .uniform(3, &self.uniform),
                ),
            );
        }

        let descriptor = self.blit_descriptor.as_ref().unwrap();
        let pipeline = &self.pipeline;

        // Render fullscreen triangle
        rcx.render(
            RenderOptions {
                label: Some("Render To Surface"),
                color_targets: &[RenderTarget::Surface],
                depth_target: None,
                clear_color: Some([0.0, 0.0, 0.0, 1.0]),
                clear_depth: None,
            },
            |mut fb| {
                fb.use_pipeline(pipeline).bind_descriptor_set(0, descriptor);
                // Draw 3 vertices for fullscreen triangle (no vertex buffer needed)
                fb.draw(0..3);
            },
        )
        .expect("failed to render post-process pass");
    }

    fn resize(&mut self, _rcx: &RenderContext, _dimensions: Dimensions) {
        // Invalidate cached descriptor - will be rebuilt in next draw() with new texture
        self.blit_descriptor = None;
    }
}
