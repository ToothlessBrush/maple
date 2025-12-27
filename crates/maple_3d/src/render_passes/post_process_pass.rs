use maple_engine::Scene;
use maple_renderer::{
    core::{
        CullMode, DescriptorBindingType, DescriptorSet, DescriptorSetLayout,
        DescriptorSetLayoutDescriptor, RenderContext, StageFlags,
        pipeline::{AlphaMode, PipelineCreateInfo, RenderPipeline},
        texture::{FilterMode, Sampler, SamplerOptions, TextureMode},
    },
    render_graph::{
        graph::RenderGraphContext,
        node::{DepthTarget, RenderNode, RenderNodeContext, RenderNodeDescriptor, RenderTarget},
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
pub struct PostProcessPass {
    blit_layout: Option<DescriptorSetLayout>,
    blit_descriptor: Option<DescriptorSet>,
    sampler: Option<Sampler>,
    pipeline: Option<RenderPipeline>,
}

impl Default for PostProcessPass {
    fn default() -> Self {
        Self {
            blit_layout: None,
            blit_descriptor: None,
            sampler: None,
            pipeline: None,
        }
    }
}

impl RenderNode for PostProcessPass {
    fn setup(
        &mut self,
        render_ctx: &RenderContext,
        _graph_ctx: &mut RenderGraphContext,
    ) -> RenderNodeDescriptor {
        // Load fullscreen triangle shaders
        let shader = render_ctx.create_shader_pair(maple_renderer::core::ShaderPair::Wgsl {
            vert: include_str!("../../res/shaders/post_process/blit.vert.wgsl"),
            frag: include_str!("../../res/shaders/post_process/blit.frag.wgsl"),
        });

        // Create descriptor layout for texture + sampler binding
        let blit_layout = render_ctx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
            label: Some("post_process_blit_layout"),
            visibility: StageFlags::FRAGMENT,
            layout: &[
                DescriptorBindingType::TextureView,  // Binding 0: resolved color texture
                DescriptorBindingType::Sampler,      // Binding 1: linear sampler
            ],
        });

        // Create sampler once (never changes)
        let sampler = render_ctx.create_sampler(SamplerOptions {
            mode_u: TextureMode::ClampToEdge,
            mode_v: TextureMode::ClampToEdge,
            mode_w: TextureMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            compare: None,
        });

        self.blit_layout = Some(blit_layout.clone());
        self.sampler = Some(sampler);

        // Create pipeline
        let pipeline_layout = render_ctx.create_pipeline_layout(&[blit_layout.clone()]);

        let depth_mode = maple_renderer::render_graph::node::DepthMode::None;

        let surface_format = render_ctx.surface_format();

        let pipeline = render_ctx.create_pipeline(PipelineCreateInfo {
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

        self.pipeline = Some(pipeline);

        RenderNodeDescriptor {
            shader,
            descriptor_set_layouts: vec![blit_layout],
            target: vec![RenderTarget::Surface],  // Render directly to surface
            depth: DepthTarget::None,              // No depth testing for fullscreen blit
            cull_mode: CullMode::None,             // No culling for fullscreen triangle
        }
    }

    fn draw(
        &mut self,
        render_ctx: &RenderContext,
        node_ctx: &mut RenderNodeContext,
        graph_ctx: &mut RenderGraphContext,
        _scene: &Scene,
    ) {
        // Get the resolved color texture from graph context
        let Some(resolved_texture) = graph_ctx.get_shared_resource::<maple_renderer::core::texture::Texture>("resolved_color_texture") else {
            maple_engine::utils::Debug::print_once("Missing resolved_color_texture in post-process pass");
            return;
        };

        // Build descriptor once (invalidated on resize)
        if self.blit_descriptor.is_none() {
            let layout = self.blit_layout.as_ref().unwrap();
            let sampler = self.sampler.as_ref().unwrap();

            self.blit_descriptor = Some(render_ctx.build_descriptor_set(
                DescriptorSet::builder(layout)
                    .texture_view(0, &resolved_texture.create_view())
                    .sampler(1, sampler),
            ));
        }

        let descriptor = self.blit_descriptor.as_ref().unwrap();
        let Some(pipeline) = &self.pipeline else {
            return;
        };

        // Render fullscreen triangle
        render_ctx
            .render(node_ctx, |mut fb| {
                fb.use_pipeline(pipeline)
                    .bind_descriptor_set(0, descriptor);
                // Draw 3 vertices for fullscreen triangle (no vertex buffer needed)
                fb.draw(0..3);
            })
            .expect("failed to render post-process pass");
    }

    fn resize(
        &mut self,
        _render_ctx: &RenderContext,
        _node_ctx: &mut RenderNodeContext,
        _dimensions: [u32; 2],
    ) {
        // Invalidate cached descriptor - will be rebuilt in next draw() with new texture
        self.blit_descriptor = None;
    }
}
