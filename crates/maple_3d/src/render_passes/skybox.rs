use maple_engine::GameContext;
use maple_renderer::{
    core::{
        Buffer, CullMode, DepthCompare, DepthMode, DepthStencilOptions, DescriptorSet,
        DescriptorSetLayout, DescriptorSetLayoutDescriptor, Frame, GraphicsShader, RenderContext,
        RenderTarget, StageFlags,
        context::RenderOptions,
        descriptor_set::DescriptorBindingType,
        pipeline::{AlphaMode, PipelineCreateInfo, RenderPipeline},
        texture::{FilterMode, Sampler, SamplerOptions, TextureFormat, TextureMode},
    },
    render_graph::{
        graph::{RenderGraphContext, Stage},
        node::RenderNode,
    },
};

use crate::{
    nodes::{
        camera::{Camera3D, Camera3DBufferData},
        environment::Environment,
    },
    render_passes::{
        environment::{EnvironmentMap, GeneratedEnviornmentTextures},
        scene_textures::{MsaaColorTexture, MsaaDepth, MsaaResolveTexture},
    },
};

pub struct SkyboxRender {
    pipeline: RenderPipeline,
    camera_buffer: Buffer<Camera3DBufferData>,
    sampler: Sampler,
    camera_layout: DescriptorSetLayout,
    texture_layout: DescriptorSetLayout,
}

impl SkyboxRender {}

impl RenderNode for SkyboxRender {
    fn label() -> &'static str
    where
        Self: Sized,
    {
        "Skybox"
    }

    fn stage(&self) -> Stage {
        Stage::PrePass
    }
    fn setup(rcx: &RenderContext, _gcx: &mut RenderGraphContext) -> Self {
        let shader = GraphicsShader {
            vertex: rcx
                .device()
                .compile_shader(include_str!("./skybox.vert.wgsl").into())
                .expect("skybox shader to compile"),
            fragment: rcx
                .device()
                .compile_shader(include_str!("./skybox.frag.wgsl").into())
                .expect("skybox fragment to compile"),
        };

        // Camera layout (group 0)
        let camera_layout =
            rcx.device()
                .create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
                    label: Some("skybox_camera_layout"),
                    visibility: StageFlags::VERTEX,
                    layout: &[DescriptorBindingType::UniformBuffer],
                });

        // Texture layout (group 1)
        let texture_layout =
            rcx.device()
                .create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
                    label: Some("skybox_texture_layout"),
                    visibility: StageFlags::FRAGMENT,
                    layout: &[
                        DescriptorBindingType::TextureViewCube { filterable: true },
                        DescriptorBindingType::Sampler { filtering: true },
                    ],
                });

        let camera_buffer = rcx
            .device()
            .create_uniform_buffer(&crate::nodes::camera::Camera3DBufferData::default());

        let pipeline_layout = rcx
            .device()
            .create_pipeline_layout(&[camera_layout.clone(), texture_layout.clone()]);

        // Create pipeline with depth comparison LessEqual so skybox renders at depth 1.0
        let pipeline = rcx.device().create_pipeline(PipelineCreateInfo {
            label: Some("Skybox"),
            layout: pipeline_layout,
            shader: shader.clone(),
            color_formats: &[TextureFormat::RGBA16Float],
            depth: DepthMode::Texture(DepthStencilOptions {
                format: TextureFormat::Depth32,
                compare: DepthCompare::LessEqual,
                write_enabled: false,
                depth_bias: None,
            }),
            cull_mode: CullMode::None,
            alpha_mode: AlphaMode::Opaque,
            sample_count: 4, // TODO: Match main pass MSAA from config
            vertex_buffer_layout: None,
        });

        let sampler = rcx.device().create_sampler(SamplerOptions {
            mode_u: TextureMode::ClampToEdge,
            mode_v: TextureMode::ClampToEdge,
            mode_w: TextureMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            compare: None,
        });

        Self {
            pipeline,
            camera_buffer,
            sampler,
            camera_layout,
            texture_layout,
        }
    }

    fn draw(
        &mut self,
        rcx: &RenderContext,
        frame: &mut Frame,
        graph_ctx: &mut RenderGraphContext,
        game_ctx: &GameContext,
    ) {
        let (
            Some(MsaaColorTexture(msaa_color)),
            Some(MsaaResolveTexture(resolved_color)),
            Some(MsaaDepth(msaa_depth)),
            Some(environment_textures),
        ) = (
            graph_ctx.get_shared_resource(),
            graph_ctx.get_shared_resource(),
            graph_ctx.get_shared_resource(),
            graph_ctx.get_shared_resource::<EnvironmentMap>(),
        )
        else {
            return;
        };

        let scene = &game_ctx.scene;
        // Get active camera
        let cameras = scene.collect::<Camera3D>();
        let Some(camera) = cameras
            .iter()
            .filter(|c| c.get_ref().is_active)
            .max_by_key(|c| c.get_ref().priority)
        else {
            return;
        };

        // Get environment node
        let environments = scene.collect::<Environment>();
        let Some(_environment) = environments.first() else {
            // No environment, no skybox to render
            return;
        };

        // Get the cubemap from the environment render pass
        let GeneratedEnviornmentTextures {
            cubemap,
            ibl_specular: _,
            ibl_irradiance: _,
            brdf_lut: _,
        } = environments
            .first()
            .and_then(|env| environment_textures.get(env.get_ref().hdri_source.id()))
            .cloned()
            .unwrap_or_else(|| {
                let default_textures = rcx.get_default_texture();
                GeneratedEnviornmentTextures {
                    cubemap: default_textures.prefilter_cubemap.clone(),
                    ibl_specular: default_textures.prefilter_cubemap.clone(),
                    ibl_irradiance: default_textures.irradiance_cubemap.clone(),
                    brdf_lut: default_textures.brdf_lut.clone(),
                }
            });

        // Update camera buffer
        rcx.queue().write_buffer(
            &self.camera_buffer,
            &camera.get_ref().get_buffer_data(rcx.aspect_ratio()),
        );

        // Build descriptor sets
        let camera_set = rcx.device().build_descriptor_set(
            DescriptorSet::builder(&self.camera_layout).uniform(0, &self.camera_buffer),
        );

        let texture_set = rcx.device().build_descriptor_set(
            DescriptorSet::builder(&self.texture_layout)
                .texture_view(0, &cubemap.create_view())
                .sampler(1, &self.sampler),
        );

        // Render the skybox with MSAA + resolve
        frame.render(
            RenderOptions {
                label: Some("Skybox Pass"),
                color_targets: &[RenderTarget::MultiSampled {
                    texture: msaa_color.create_view(),
                    resolve: resolved_color.create_view(),
                }],
                depth_target: Some(&msaa_depth.create_view()),
                clear_color: Some([0.1, 0.1, 0.1, 1.0]),
                clear_depth: Some(1.0),
            },
            |mut fb| {
                fb.use_pipeline(&self.pipeline)
                    .bind_descriptor_set(0, &camera_set)
                    .bind_descriptor_set(1, &texture_set)
                    .draw(0..36, 0); // 36 vertices for a cube
            },
        )
    }
}
