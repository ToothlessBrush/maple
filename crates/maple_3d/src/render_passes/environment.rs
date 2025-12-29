use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use maple_engine::Scene;
use maple_renderer::{
    core::{
        Buffer, CullMode, DepthCompare, DescriptorSetBuilder, GraphicsShader, RenderContext,
        ShaderPair, StageFlags,
        context::RenderOptions,
        descriptor_set::{
            DescriptorBindingType, DescriptorSet, DescriptorSetLayout,
            DescriptorSetLayoutDescriptor,
        },
        pipeline::{AlphaMode, PipelineCreateInfo, RenderPipeline},
        texture::{
            CubeFace, Sampler, Texture, TextureCreateInfo, TextureCube, TextureCubeArray,
            TextureFormat, TextureUsage,
        },
    },
    render_graph::{
        graph::{NodeLabel, RenderGraphContext},
        node::{DepthMode, RenderNode, RenderTarget},
    },
};

use crate::{
    components::material::MaterialProperties,
    nodes::{environment::Environment, mesh::Mesh3D, point_light::PointLight},
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct EquirectUniforms {
    face_index: u32,
    _padding: [u32; 15],
}

pub struct EnvironmentLabel;

impl NodeLabel for EnvironmentLabel {}

#[derive(Default)]
pub struct EnvironmentRender {
    // Render pipeline
    pipeline: Option<RenderPipeline>,

    uniform_buffer: Option<Buffer<EquirectUniforms>>,

    sampler: Option<Sampler>,

    layout: Option<DescriptorSetLayout>,

    cubemap: Option<TextureCube>,
}

impl RenderNode for EnvironmentRender {
    fn setup(&mut self, render_ctx: &RenderContext, _graph_ctx: &mut RenderGraphContext) {
        let shader = render_ctx.create_shader_pair(ShaderPair::Wgsl {
            vert: include_str!("../../res/shaders/flat_to_cube/flat_to_cube.vert.wgsl"),
            frag: include_str!("../../res/shaders/flat_to_cube/flat_to_cube.frag.wgsl"),
        });

        let layout = render_ctx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
            label: Some("EnvironmentToCube"),
            visibility: StageFlags::VERTEX | StageFlags::FRAGMENT,
            layout: &[
                DescriptorBindingType::TextureView { filterable: false },
                DescriptorBindingType::Sampler { filtering: false },
                DescriptorBindingType::UniformBuffer,
            ],
        });

        let uniform_buffer = render_ctx.create_uniform_buffer(&EquirectUniforms {
            face_index: 0,
            _padding: [0; 15],
        });

        let pipeline_layout = render_ctx.create_pipeline_layout(&[layout.clone()]);

        let pipeline = render_ctx.create_pipeline(PipelineCreateInfo {
            label: Some("FlatToCube"),
            layout: pipeline_layout,
            shader: shader.clone(),
            color_formats: &[TextureFormat::RGBA16Float],
            depth: &DepthMode::None,
            cull_mode: CullMode::None,
            alpha_mode: AlphaMode::Opaque,
            sample_count: 1,
            use_vertex_buffer: false,
        });

        let sampler = render_ctx.create_sampler(maple_renderer::core::texture::SamplerOptions {
            mode_u: maple_renderer::core::texture::TextureMode::Repeat,
            mode_v: maple_renderer::core::texture::TextureMode::ClampToEdge,
            mode_w: maple_renderer::core::texture::TextureMode::ClampToEdge,
            mag_filter: maple_renderer::core::texture::FilterMode::Nearest,
            min_filter: maple_renderer::core::texture::FilterMode::Nearest,
            compare: None,
        });

        self.pipeline = Some(pipeline);
        self.uniform_buffer = Some(uniform_buffer);
        self.sampler = Some(sampler);
        self.layout = Some(layout);
    }

    fn draw(
        &mut self,
        render_ctx: &RenderContext,
        graph_ctx: &mut RenderGraphContext,
        scene: &Scene,
    ) {
        // scene should only have 1 environment node
        let environments = scene.collect_items::<Environment>();

        let Some(environment) = environments.first() else {
            return;
        };

        if self.cubemap.is_none() {
            let cubemap = render_ctx.create_texture_cube(
                maple_renderer::core::texture::TextureCubeCreateInfo {
                    label: Some("environment cubemap"),
                    size: 512,
                    format: TextureFormat::RGBA16Float,
                    usage: TextureUsage::TEXTURE_BINDING | TextureUsage::RENDER_ATTACHMENT,
                    mip_level: 1,
                },
            );
            self.cubemap = Some(cubemap);
        }

        let hdri = environment.get_hdri_texture(render_ctx);

        let descrptor = render_ctx.build_descriptor_set(
            DescriptorSet::builder(self.layout.as_ref().unwrap())
                .texture_view(0, &hdri.create_view())
                .sampler(1, self.sampler.as_ref().unwrap())
                .uniform(2, self.uniform_buffer.as_ref().unwrap()),
        );

        let pipeline = self.pipeline.as_ref().unwrap();
        let uniform_buffer = self.uniform_buffer.as_ref().unwrap();
        let cubemap = self.cubemap.as_ref().unwrap();

        // Share the cubemap with other render passes (like skybox)
        graph_ctx.add_shared_resource("environment_cubemap", cubemap.clone());

        for face_idx in 0..6 {
            let face = match face_idx {
                0 => CubeFace::PositiveX,
                1 => CubeFace::NegativeX,
                2 => CubeFace::PositiveY,
                3 => CubeFace::NegativeY,
                4 => CubeFace::PositiveZ,
                5 => CubeFace::NegativeZ,
                _ => unreachable!(),
            };

            let uniform = EquirectUniforms {
                face_index: face_idx,
                _padding: [0; 15],
            };

            render_ctx.write_buffer(uniform_buffer, &uniform);

            let face_texture = cubemap.create_face_texture(face, 0);

            render_ctx
                .render(
                    RenderOptions {
                        color_targets: &[RenderTarget::Texture(face_texture)],
                        depth_target: None,
                        clear_color: Some([0.0, 0.0, 0.0, 1.0]),
                    },
                    |mut fb| {
                        fb.use_pipeline(pipeline)
                            .bind_descriptor_set(0, &descrptor)
                            .draw(0..3);
                    },
                )
                .expect("failed to draw cubemap");
        }
    }
}
