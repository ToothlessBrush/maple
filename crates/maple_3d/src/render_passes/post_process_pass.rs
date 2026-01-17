use std::slice;

use maple_engine::Scene;
use maple_renderer::{
    core::{
        CullMode, DescriptorBindingType, DescriptorSet, DescriptorSetLayout,
        DescriptorSetLayoutDescriptor, RenderContext, StageFlags,
        context::RenderOptions,
        pipeline::{AlphaMode, PipelineCreateInfo, RenderPipeline},
        texture::{FilterMode, Sampler, SamplerOptions, TextureMode},
    },
    render_graph::{
        graph::RenderGraphContext,
        node::{DepthMode, RenderNode, RenderTarget},
    },
};

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
                DescriptorBindingType::Sampler { filtering: true }, // Binding 1: linear sampler
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

        Self {
            blit_layout,
            blit_descriptor: None,
            sampler,
            pipeline,
        }
    }
}

impl RenderNode for CompositePass {
    fn draw(&mut self, rcx: &RenderContext, graph_ctx: &mut RenderGraphContext, _scene: &Scene) {
        // Get the resolved color texture from graph context
        let Some(resolved_texture) = graph_ctx
            .get_shared_resource::<maple_renderer::core::texture::Texture>(
                "resolved_color_texture",
            )
        else {
            return;
        };

        // Build descriptor once (invalidated on resize)
        if self.blit_descriptor.is_none() {
            let layout = &self.blit_layout;
            let sampler = &self.sampler;

            self.blit_descriptor = Some(
                rcx.build_descriptor_set(
                    DescriptorSet::builder(layout)
                        .texture_view(0, &resolved_texture.create_view())
                        .sampler(1, sampler),
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

    fn resize(&mut self, _rcx: &RenderContext, _dimensions: [u32; 2]) {
        // Invalidate cached descriptor - will be rebuilt in next draw() with new texture
        self.blit_descriptor = None;
    }
}
