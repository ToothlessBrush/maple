use maple_engine::{Scene, utils::Debug};
use maple_renderer::{
    core::{
        Buffer, CullMode, DepthCompare, DepthStencilOptions, DescriptorSet,
        DescriptorSetLayoutDescriptor, RenderContext, ShaderPair, StageFlags,
        context::RenderOptions,
        descriptor_set::DescriptorBindingType,
        pipeline::{AlphaMode, PipelineCreateInfo, RenderPipeline},
        texture::{Sampler, SamplerOptions, Texture, TextureCube, TextureFormat},
    },
    render_graph::{
        graph::{NodeLabel, RenderGraphContext},
        node::{DepthMode, RenderNode, RenderTarget},
    },
};

use crate::nodes::{camera::Camera3D, environment::Environment};

pub struct SkyboxLabel;
impl NodeLabel for SkyboxLabel {}

#[derive(Default)]
pub struct SkyboxRender {
    pipeline: Option<RenderPipeline>,
    camera_buffer: Option<Buffer<crate::nodes::camera::Camera3DBufferData>>,
    sampler: Option<Sampler>,
    camera_layout: Option<maple_renderer::core::descriptor_set::DescriptorSetLayout>,
    texture_layout: Option<maple_renderer::core::descriptor_set::DescriptorSetLayout>,
}

impl RenderNode for SkyboxRender {
    fn setup(&mut self, render_ctx: &RenderContext, _graph_ctx: &mut RenderGraphContext) {
        let shader = render_ctx.create_shader_pair(ShaderPair::Wgsl {
            vert: include_str!("../../res/shaders/default/skybox.vert.wgsl"),
            frag: include_str!("../../res/shaders/default/skybox.frag.wgsl"),
        });

        // Camera layout (group 0)
        let camera_layout =
            render_ctx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
                label: Some("skybox_camera_layout"),
                visibility: StageFlags::VERTEX,
                layout: &[DescriptorBindingType::UniformBuffer],
            });

        // Texture layout (group 1)
        let texture_layout =
            render_ctx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
                label: Some("skybox_texture_layout"),
                visibility: StageFlags::FRAGMENT,
                layout: &[
                    DescriptorBindingType::TextureViewCube { filterable: true },
                    DescriptorBindingType::Sampler { filtering: true },
                ],
            });

        let camera_buffer =
            render_ctx.create_uniform_buffer(&crate::nodes::camera::Camera3DBufferData::default());

        let pipeline_layout =
            render_ctx.create_pipeline_layout(&[camera_layout.clone(), texture_layout.clone()]);

        let surface_format = render_ctx.surface_format();

        // Create a placeholder depth texture for pipeline creation
        let dimensions = render_ctx.surface_size();
        let placeholder_depth =
            render_ctx.create_texture(maple_renderer::core::texture::TextureCreateInfo {
                label: Some("skybox_placeholder_depth"),
                width: dimensions.0,
                height: dimensions.1,
                format: maple_renderer::core::texture::TextureFormat::Depth32,
                usage: maple_renderer::core::texture::TextureUsage::RENDER_ATTACHMENT,
                sample_count: 4,
                mip_level: 1,
            });

        // Create pipeline with depth comparison LessEqual so skybox renders at depth 1.0
        let pipeline = render_ctx.create_pipeline(PipelineCreateInfo {
            label: Some("Skybox"),
            layout: pipeline_layout,
            shader: shader.clone(),
            color_formats: &[surface_format],
            depth: &DepthMode::Texture(DepthStencilOptions {
                format: TextureFormat::Depth32,
                compare: DepthCompare::LessEqual,
                write_enabled: false,
                depth_bias: None,
            }),
            cull_mode: CullMode::None,
            alpha_mode: AlphaMode::Opaque,
            sample_count: 4, // Match main pass MSAA
            use_vertex_buffer: false,
        });

        let sampler = render_ctx.create_sampler(SamplerOptions {
            mode_u: maple_renderer::core::texture::TextureMode::ClampToEdge,
            mode_v: maple_renderer::core::texture::TextureMode::ClampToEdge,
            mode_w: maple_renderer::core::texture::TextureMode::ClampToEdge,
            mag_filter: maple_renderer::core::texture::FilterMode::Linear,
            min_filter: maple_renderer::core::texture::FilterMode::Linear,
            compare: None,
        });

        self.pipeline = Some(pipeline);
        self.camera_buffer = Some(camera_buffer);
        self.sampler = Some(sampler);
        self.camera_layout = Some(camera_layout);
        self.texture_layout = Some(texture_layout);
    }

    fn draw(
        &mut self,
        render_ctx: &RenderContext,
        graph_ctx: &mut RenderGraphContext,
        scene: &Scene,
    ) {
        // Get active camera
        let cameras = scene.collect_items::<Camera3D>();
        let Some(camera) = cameras
            .iter()
            .filter(|c| c.is_active)
            .max_by_key(|c| c.priority)
        else {
            Debug::print_once("no active camera in scene for skybox");
            return;
        };

        // Get environment node
        let environments = scene.collect_items::<Environment>();
        let Some(_environment) = environments.first() else {
            // No environment, no skybox to render
            return;
        };

        // Get the cubemap from the environment render pass
        let Some(cubemap) = graph_ctx.get_shared_resource::<TextureCube>("environment_cubemap")
        else {
            Debug::print_once("No environment cubemap found in graph context");
            return;
        };

        // Get the MSAA color texture and depth texture from main pass
        let Some(msaa_color_texture) =
            graph_ctx.get_shared_resource::<Texture>("msaa_color_texture")
        else {
            Debug::print_once("No MSAA color texture found");
            return;
        };

        let Some(resolved_color_texture) =
            graph_ctx.get_shared_resource::<Texture>("resolved_color_texture")
        else {
            Debug::print_once("No resolved color texture found");
            return;
        };

        let Some(depth_texture) = graph_ctx.get_shared_resource::<Texture>("main_depth_texture")
        else {
            Debug::print_once("No main depth texture found");
            return;
        };

        // Update camera buffer
        let camera_buffer = self.camera_buffer.as_ref().unwrap();
        render_ctx.write_buffer(
            camera_buffer,
            &camera.get_buffer_data(render_ctx.aspect_ratio()),
        );

        // Build descriptor sets
        let camera_set = render_ctx.build_descriptor_set(
            DescriptorSet::builder(self.camera_layout.as_ref().unwrap()).uniform(0, camera_buffer),
        );

        let texture_set = render_ctx.build_descriptor_set(
            DescriptorSet::builder(self.texture_layout.as_ref().unwrap())
                .texture_view(0, &cubemap.create_view())
                .sampler(1, self.sampler.as_ref().unwrap()),
        );

        // Render the skybox with MSAA + resolve
        render_ctx
            .render(
                RenderOptions {
                    color_targets: &[RenderTarget::MultiSampled {
                        texture: msaa_color_texture.clone(),
                        resolve: resolved_color_texture.clone(),
                    }],
                    depth_target: Some(depth_texture),
                    clear_color: None, // Don't clear - we're rendering on top of the main pass
                },
                |mut fb| {
                    fb.use_pipeline(self.pipeline.as_ref().unwrap())
                        .bind_descriptor_set(0, &camera_set)
                        .bind_descriptor_set(1, &texture_set)
                        .draw(0..36); // 36 vertices for a cube
                },
            )
            .expect("failed to render skybox");
    }
}
