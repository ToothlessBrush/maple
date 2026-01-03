use std::slice;

use bytemuck::{Pod, Zeroable};
use maple_engine::Scene;
use maple_renderer::{
    core::{
        Buffer, ComputePipeline, ComputePipelineCreateInfo, ComputeShaderSource, CullMode,
        RenderContext, ShaderPair, StageFlags,
        context::RenderOptions,
        descriptor_set::{
            DescriptorBindingType, DescriptorSet, DescriptorSetLayout,
            DescriptorSetLayoutDescriptor,
        },
        pipeline::{AlphaMode, PipelineCreateInfo, RenderPipeline},
        texture::{
            CubeFace, Sampler, Texture, TextureCreateInfo, TextureCube, TextureCubeCreateInfo,
            TextureFormat, TextureUsage,
        },
    },
    render_graph::{
        graph::{NodeLabel, RenderGraphContext},
        node::{DepthMode, RenderNode, RenderTarget},
    },
};

use crate::nodes::environment::Environment;

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

    // Irradiance IBL
    irradiance_pipeline: Option<RenderPipeline>,
    irradiance_map: Option<TextureCube>,
    irradiance_layout: Option<DescriptorSetLayout>,
    irradiance_sampler: Option<Sampler>,

    // Specular IBL
    prefilter_map: Option<TextureCube>,
    prefilter_pipeline: Option<ComputePipeline>,
    prefilter_layout: Option<DescriptorSetLayout>,
    prefilter_sampler: Option<Sampler>,

    // BRDF LUT
    brdf_pipeline: Option<ComputePipeline>,
    brdf_texture: Option<Texture>,
    brdf_layout: Option<DescriptorSetLayout>,
}

impl RenderNode for EnvironmentRender {
    fn setup(&mut self, render_ctx: &RenderContext, _graph_ctx: &mut RenderGraphContext) {
        let shader = render_ctx.create_shader_pair(ShaderPair::Wgsl {
            vert: include_str!("../../res/shaders/environment/flat_to_cube.vert.wgsl"),
            frag: include_str!("../../res/shaders/environment/flat_to_cube.frag.wgsl"),
        });

        let irradiance_shader = render_ctx.create_shader_pair(ShaderPair::Wgsl {
            vert: include_str!("../../res/shaders/environment/irradiance.vert.wgsl"),
            frag: include_str!("../../res/shaders/environment/irradiance.frag.wgsl"),
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

        let irradiance_layout =
            render_ctx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
                label: Some("irradiance layout"),
                visibility: StageFlags::VERTEX | StageFlags::FRAGMENT,
                layout: &[
                    DescriptorBindingType::TextureViewCube { filterable: true },
                    DescriptorBindingType::Sampler { filtering: true },
                    DescriptorBindingType::UniformBuffer,
                ],
            });

        let irradiance_pipeline_layout =
            render_ctx.create_pipeline_layout(slice::from_ref(&irradiance_layout));

        let uniform_buffer = render_ctx.create_uniform_buffer(&EquirectUniforms {
            face_index: 0,
            _padding: [0; 15],
        });

        let pipeline_layout = render_ctx.create_pipeline_layout(slice::from_ref(&layout));

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

        let irradiance_pipeline = render_ctx.create_pipeline(PipelineCreateInfo {
            label: Some("irradiance generation"),
            layout: irradiance_pipeline_layout,
            shader: irradiance_shader.clone(),
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

        let irradiance_sampler =
            render_ctx.create_sampler(maple_renderer::core::texture::SamplerOptions {
                mode_u: maple_renderer::core::texture::TextureMode::Repeat,
                mode_v: maple_renderer::core::texture::TextureMode::ClampToEdge,
                mode_w: maple_renderer::core::texture::TextureMode::ClampToEdge,
                mag_filter: maple_renderer::core::texture::FilterMode::Linear,
                min_filter: maple_renderer::core::texture::FilterMode::Linear,
                compare: None,
            });

        // Prefilter compute pipeline setup
        let prefilter_shader = render_ctx.create_compute_shader(ComputeShaderSource::Wgsl(
            include_str!("../../res/shaders/environment/prefilter.wgsl"),
        ));

        let prefilter_layout =
            render_ctx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
                label: Some("prefilter layout"),
                visibility: StageFlags::COMPUTE,
                layout: &[
                    DescriptorBindingType::TextureViewCube { filterable: true },
                    DescriptorBindingType::Sampler { filtering: true },
                    DescriptorBindingType::StorageTexture2D {
                        format: TextureFormat::RGBA16Float,
                    },
                    DescriptorBindingType::UniformBuffer,
                ],
            });

        let prefilter_pipeline_layout =
            render_ctx.create_pipeline_layout(slice::from_ref(&prefilter_layout));

        let prefilter_pipeline = render_ctx.create_compute_pipeline(ComputePipelineCreateInfo {
            label: Some("prefilter specular IBL"),
            layout: prefilter_pipeline_layout,
            shader: prefilter_shader,
            entry_point: None,
        });

        let prefilter_sampler =
            render_ctx.create_sampler(maple_renderer::core::texture::SamplerOptions {
                mode_u: maple_renderer::core::texture::TextureMode::ClampToEdge,
                mode_v: maple_renderer::core::texture::TextureMode::ClampToEdge,
                mode_w: maple_renderer::core::texture::TextureMode::ClampToEdge,
                mag_filter: maple_renderer::core::texture::FilterMode::Linear,
                min_filter: maple_renderer::core::texture::FilterMode::Linear,
                compare: None,
            });

        // Prefilter compute pipeline setup
        let brdf_lut_shader = render_ctx.create_compute_shader(ComputeShaderSource::Wgsl(
            include_str!("../../res/shaders/environment/brdf_lut.wgsl"),
        ));

        let brdf_lut_layout =
            render_ctx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
                label: Some("prefilter layout"),
                visibility: StageFlags::COMPUTE,
                layout: &[DescriptorBindingType::StorageTexture2D {
                    format: TextureFormat::RG32Float,
                }],
            });

        let brdf_lut_pipeline_layout =
            render_ctx.create_pipeline_layout(slice::from_ref(&brdf_lut_layout));

        let brdf_lut_pipeline = render_ctx.create_compute_pipeline(ComputePipelineCreateInfo {
            label: Some("prefilter specular IBL"),
            layout: brdf_lut_pipeline_layout,
            shader: brdf_lut_shader,
            entry_point: None,
        });

        self.pipeline = Some(pipeline);
        self.uniform_buffer = Some(uniform_buffer);
        self.sampler = Some(sampler);
        self.layout = Some(layout);
        self.irradiance_pipeline = Some(irradiance_pipeline);
        self.irradiance_layout = Some(irradiance_layout);
        self.irradiance_sampler = Some(irradiance_sampler);
        self.prefilter_pipeline = Some(prefilter_pipeline);
        self.prefilter_layout = Some(prefilter_layout);
        self.prefilter_sampler = Some(prefilter_sampler);
        self.brdf_layout = Some(brdf_lut_layout);
        self.brdf_pipeline = Some(brdf_lut_pipeline);
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

        if self.cubemap.is_some() && self.irradiance_map.is_some() && self.prefilter_map.is_some() {
            return;
        }

        let cubemap = render_ctx.create_texture_cube(TextureCubeCreateInfo {
            label: Some("environment cubemap"),
            size: 2048,
            format: TextureFormat::RGBA16Float,
            usage: TextureUsage::TEXTURE_BINDING | TextureUsage::RENDER_ATTACHMENT | TextureUsage::STORAGE_BINDING,
            mip_level: 12, // log2(2048) + 1 = 12 mip levels
        });
        self.cubemap = Some(cubemap);

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

        // cubemap generation
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

            let face_view = cubemap.create_face_view(face, 0);

            render_ctx
                .render(
                    RenderOptions {
                        color_targets: &[RenderTarget::Texture(face_view)],
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

        // Generate mipmaps for the cubemap
        render_ctx.generate_cubemap_mipmaps(cubemap, 12);

        let irradiance_map = render_ctx.create_texture_cube(TextureCubeCreateInfo {
            label: Some("irradiance cubemap"),
            size: 32,
            format: TextureFormat::RGBA16Float,
            usage: TextureUsage::TEXTURE_BINDING | TextureUsage::RENDER_ATTACHMENT,
            mip_level: 1,
        });
        self.irradiance_map = Some(irradiance_map);

        let irradiance_pipeline = self.irradiance_pipeline.as_ref().unwrap();
        let irradiance_map = self.irradiance_map.as_ref().unwrap();

        graph_ctx.add_shared_resource("irradiance_cubemap", irradiance_map.clone());

        // irradiance_map_generation
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

            let irradiance_descritor = render_ctx.build_descriptor_set(
                DescriptorSet::builder(self.irradiance_layout.as_ref().unwrap())
                    .texture_view(0, &cubemap.create_view())
                    .sampler(1, self.irradiance_sampler.as_ref().unwrap())
                    .uniform(2, uniform_buffer),
            );

            let face_view = irradiance_map.create_face_view(face, 0);

            render_ctx
                .render(
                    RenderOptions {
                        color_targets: &[RenderTarget::Texture(face_view)],
                        depth_target: None,
                        clear_color: Some([0.0, 0.0, 0.0, 1.0]),
                    },
                    |mut fb| {
                        fb.use_pipeline(irradiance_pipeline)
                            .bind_descriptor_set(0, &irradiance_descritor)
                            .draw(0..3);
                    },
                )
                .expect("failed to draw irradiacne map");
        }

        // Prefiltered specular map generation
        let max_mip_levels = 5u32;
        let prefilter_map = render_ctx.create_texture_cube(TextureCubeCreateInfo {
            label: Some("prefilter specular map"),
            size: 512,
            format: TextureFormat::RGBA16Float,
            usage: TextureUsage::TEXTURE_BINDING | TextureUsage::STORAGE_BINDING,
            mip_level: max_mip_levels,
        });
        self.prefilter_map = Some(prefilter_map);

        let prefilter_pipeline = self.prefilter_pipeline.as_ref().unwrap();
        let prefilter_map = self.prefilter_map.as_ref().unwrap();

        graph_ctx.add_shared_resource("prefilter_cubemap", prefilter_map.clone());

        // Generate each mip level with increasing roughness
        for mip in 0..max_mip_levels {
            let roughness = mip as f32 / (max_mip_levels - 1) as f32;
            let mip_size = 512u32 >> mip;

            for face_idx in 0..6 {
                let face = match face_idx {
                    0 => CubeFace::PositiveX,
                    1 => CubeFace::NegativeX,
                    2 => CubeFace::NegativeY,
                    3 => CubeFace::PositiveY,
                    4 => CubeFace::PositiveZ,
                    5 => CubeFace::NegativeZ,
                    _ => unreachable!(),
                };

                // Update uniforms with roughness
                #[repr(C)]
                #[derive(Clone, Copy, Pod, Zeroable)]
                struct PrefilterUniforms {
                    roughness: f32,
                    face: u32,
                    mip_level: u32,
                    resolution: f32,
                }

                let prefilter_uniform = PrefilterUniforms {
                    roughness,
                    face: face_idx,
                    mip_level: mip,
                    resolution: 2048.0, // Source cubemap resolution
                };

                let prefilter_uniform_buffer = render_ctx.create_uniform_buffer(&prefilter_uniform);

                let face_view = prefilter_map.create_face_view(face, mip);

                let prefilter_descriptor = render_ctx.build_descriptor_set(
                    DescriptorSet::builder(self.prefilter_layout.as_ref().unwrap())
                        .texture_view(0, &cubemap.create_view())
                        .sampler(1, self.prefilter_sampler.as_ref().unwrap())
                        .texture_view(2, &face_view)
                        .uniform(3, &prefilter_uniform_buffer),
                );

                let workgroup_size = 8u32;
                let dispatch_x = mip_size.div_ceil(workgroup_size);
                let dispatch_y = mip_size.div_ceil(workgroup_size);

                render_ctx.compute(Some("prefilter specular IBL"), |mut cb| {
                    cb.use_pipeline(prefilter_pipeline)
                        .bind_descriptor_set(0, &prefilter_descriptor)
                        .dispatch(dispatch_x, dispatch_y, 1);
                });
            }
        }

        let brdf_texture_size = 512;

        let brdf_texture = render_ctx.create_texture(TextureCreateInfo {
            label: Some("brdf lut"),
            width: brdf_texture_size,
            height: brdf_texture_size,
            format: TextureFormat::RG32Float,
            usage: TextureUsage::TEXTURE_BINDING | TextureUsage::STORAGE_BINDING,
            mip_level: 1,
            sample_count: 1,
        });
        self.brdf_texture = Some(brdf_texture.clone());
        graph_ctx.add_shared_resource("brdf_lut", brdf_texture.clone());

        let brdf_pipeline = self.brdf_pipeline.as_ref().unwrap();
        let brdf_layout = self.brdf_layout.as_ref().unwrap();
        let brdf_descriptor = render_ctx.build_descriptor_set(
            DescriptorSet::builder(brdf_layout).texture_view(0, &brdf_texture.create_view()),
        );

        let workgroup_size = 8;
        let dispatch_x = brdf_texture_size.div_ceil(workgroup_size);
        let dispatch_y = brdf_texture_size.div_ceil(workgroup_size);

        render_ctx.compute(Some("brdf_lut_generation"), |mut cb| {
            cb.use_pipeline(brdf_pipeline)
                .bind_descriptor_set(0, &brdf_descriptor)
                .dispatch(dispatch_x, dispatch_y, 1);
        });
    }
}
