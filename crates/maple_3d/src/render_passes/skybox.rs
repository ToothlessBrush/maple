use maple_engine::{Scene, utils::Debug};
use maple_renderer::{
    core::{
        Buffer, CullMode, DepthCompare, DepthStencilOptions, DescriptorSet, DescriptorSetLayout,
        DescriptorSetLayoutDescriptor, RenderContext, ShaderPair, StageFlags,
        context::RenderOptions,
        descriptor_set::DescriptorBindingType,
        pipeline::{AlphaMode, PipelineCreateInfo, RenderPipeline},
        texture::{
            FilterMode, Sampler, SamplerOptions, Texture, TextureCube, TextureFormat, TextureMode,
        },
    },
    render_graph::{
        graph::RenderGraphContext,
        node::{DepthMode, RenderNode, RenderTarget},
    },
};

use crate::nodes::{
    camera::{Camera3D, Camera3DBufferData},
    environment::Environment,
};

pub struct SkyboxRender {
    pipeline: RenderPipeline,
    camera_buffer: Buffer<Camera3DBufferData>,
    sampler: Sampler,
    camera_layout: DescriptorSetLayout,
    texture_layout: DescriptorSetLayout,
}

impl SkyboxRender {
    pub fn setup(rcx: &RenderContext, _gcx: &mut RenderGraphContext) -> Self {
        let shader = rcx.create_shader_pair(ShaderPair::Wgsl {
            vert: include_str!("../../res/shaders/default/skybox.vert.wgsl"),
            frag: include_str!("../../res/shaders/default/skybox.frag.wgsl"),
        });

        // Camera layout (group 0)
        let camera_layout = rcx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
            label: Some("skybox_camera_layout"),
            visibility: StageFlags::VERTEX,
            layout: &[DescriptorBindingType::UniformBuffer],
        });

        // Texture layout (group 1)
        let texture_layout = rcx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
            label: Some("skybox_texture_layout"),
            visibility: StageFlags::FRAGMENT,
            layout: &[
                DescriptorBindingType::TextureViewCube { filterable: true },
                DescriptorBindingType::Sampler { filtering: true },
            ],
        });

        let camera_buffer =
            rcx.create_uniform_buffer(&crate::nodes::camera::Camera3DBufferData::default());

        let pipeline_layout =
            rcx.create_pipeline_layout(&[camera_layout.clone(), texture_layout.clone()]);

        let surface_format = rcx.surface_format();

        // Create pipeline with depth comparison LessEqual so skybox renders at depth 1.0
        let pipeline = rcx.create_pipeline(PipelineCreateInfo {
            label: Some("Skybox"),
            layout: pipeline_layout,
            shader: shader.clone(),
            color_formats: &[TextureFormat::RGBA16Float],
            depth: &DepthMode::Texture(DepthStencilOptions {
                format: TextureFormat::Depth32,
                compare: DepthCompare::LessEqual,
                write_enabled: false,
                depth_bias: None,
            }),
            cull_mode: CullMode::None,
            alpha_mode: AlphaMode::Opaque,
            sample_count: 4, // TODO: Match main pass MSAA from config
            use_vertex_buffer: false,
        });

        let sampler = rcx.create_sampler(SamplerOptions {
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
}

impl RenderNode for SkyboxRender {
    fn draw(&mut self, rcx: &RenderContext, gcx: &mut RenderGraphContext, scene: &Scene) {
        // Get active camera
        let cameras = scene.collect::<Camera3D>();
        let Some(camera) = cameras
            .iter()
            .filter(|c| c.read().is_active)
            .max_by_key(|c| c.read().priority)
        else {
            Debug::print_once("no active camera in scene for skybox");
            return;
        };

        // Get environment node
        let environments = scene.collect::<Environment>();
        let Some(_environment) = environments.first() else {
            // No environment, no skybox to render
            return;
        };

        // Get the cubemap from the environment render pass
        let Some(cubemap) = gcx.get_shared_resource::<TextureCube>("environment_cubemap") else {
            Debug::print_once("No environment cubemap found in graph context");
            return;
        };

        // Get the MSAA color texture and depth texture from main pass
        let Some(msaa_color_texture) = gcx.get_shared_resource::<Texture>("msaa_color_texture")
        else {
            Debug::print_once("No MSAA color texture found");
            return;
        };

        let Some(resolved_color_texture) =
            gcx.get_shared_resource::<Texture>("resolved_color_texture")
        else {
            Debug::print_once("No resolved color texture found");
            return;
        };

        let Some(depth_texture) = gcx.get_shared_resource::<Texture>("main_depth_texture") else {
            Debug::print_once("No main depth texture found");
            return;
        };

        // Update camera buffer
        rcx.write_buffer(
            &self.camera_buffer,
            &camera.read().get_buffer_data(rcx.aspect_ratio()),
        );

        // Build descriptor sets
        let camera_set = rcx.build_descriptor_set(
            DescriptorSet::builder(&self.camera_layout).uniform(0, &self.camera_buffer),
        );

        let texture_set = rcx.build_descriptor_set(
            DescriptorSet::builder(&self.texture_layout)
                .texture_view(0, &cubemap.create_view())
                .sampler(1, &self.sampler),
        );

        // Render the skybox with MSAA + resolve
        rcx.render(
            RenderOptions {
                label: Some("Skybox Pass"),
                color_targets: &[RenderTarget::MultiSampled {
                    texture: msaa_color_texture.create_view(),
                    resolve: resolved_color_texture.create_view(),
                }],
                depth_target: Some(&depth_texture.create_view()),
                clear_color: Some([0.1, 0.1, 0.1, 1.0]),
            },
            |mut fb| {
                fb.use_pipeline(&self.pipeline)
                    .bind_descriptor_set(0, &camera_set)
                    .bind_descriptor_set(1, &texture_set)
                    .draw(0..36); // 36 vertices for a cube
            },
        )
        .expect("failed to render skybox");
    }
}
