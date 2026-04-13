use bytemuck::{Pod, Zeroable};
use maple_engine::GameContext;
use maple_renderer::{
    core::{
        Buffer, CullMode, DepthCompare, DepthStencilOptions, DescriptorBindingType, DescriptorSet,
        DescriptorSetLayoutDescriptor, RenderContext, StageFlags,
        context::{Dimensions, RenderOptions},
        descriptor_set::DescriptorSetLayout,
        pipeline::{AlphaMode as PipelineAlphaMode, PipelineCreateInfo, RenderPipeline},
        texture::{
            FilterMode, Sampler, SamplerOptions, Texture, TextureCube, TextureFormat, TextureMode,
        },
    },
    render_graph::{
        graph::RenderGraphContext,
        node::{DepthMode, RenderNode, RenderTarget},
    },
};

struct SceneDescriptor {
    pub layout: DescriptorSetLayout,
    pub scene_buffer: Buffer<SceneData>,
    pub camera_data_buffer: Buffer<Camera3DBufferData>,
    pub irradiance_sampler: Sampler,
    pub prefilter_sampler: Sampler,
    pub brdf_lut_sampler: Sampler,
}

struct MainPipelines {
    pub opaque: RenderPipeline,
    pub blend: RenderPipeline,
}

#[derive(Default, Debug, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
struct SceneData {
    background_color: [f32; 4],
    ambient: f32,
    ibl_strength: f32,
    _padding: [f32; 2],
}

impl SceneData {
    pub fn ambient(mut self, ambient: f32) -> Self {
        self.ambient = ambient;
        self
    }

    pub fn ibl_strength(mut self, strength: f32) -> Self {
        self.ibl_strength = strength;
        self
    }
}

use crate::{
    components::material::{AlphaMode, MaterialProperties},
    math::Frustum,
    nodes::{
        camera::{Camera3D, Camera3DBufferData},
        directional_light::{DirectionalLight, DirectionalLightBuffer},
        environment::Environment,
        mesh::Mesh3D,
        point_light::{PointLight, PointLightBuffer},
    },
    render_passes::shadow_resource::ShadowResource,
};

struct TextureCache {
    msaa_color: Texture,
    resolved_color: Texture,
    msaa_normal: Texture,
    resolved_normal: Texture,
    msaa_depth: Texture,
}

pub struct MainPass {
    scene_data: SceneDescriptor,
    pipelines: MainPipelines,
    // Render targets cached so we dont need to fetch from graph every frame (maybe this is useless)
    texture_cache: Option<TextureCache>,
}

impl MainPass {
    pub fn setup(rcx: &RenderContext, _gcx: &mut RenderGraphContext) -> Self {
        // shader
        let shader = rcx.create_shader_pair(maple_renderer::core::ShaderPair::Wgsl {
            vert: include_str!("../../res/shaders/default/default.vert.wgsl"),
            frag: include_str!("../../res/shaders/default/default.frag.wgsl"),
        });

        // layouts
        let material_layout = MaterialProperties::layout(rcx).clone();
        let mesh_layout = Mesh3D::layout(rcx).clone();
        let scene_layout = rcx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
            label: Some("scene layout"),
            visibility: StageFlags::VERTEX | StageFlags::FRAGMENT,
            layout: &[
                DescriptorBindingType::UniformBuffer,
                DescriptorBindingType::UniformBuffer,
                DescriptorBindingType::TextureViewCube { filterable: true },
                DescriptorBindingType::Sampler { filtering: true },
                DescriptorBindingType::TextureViewCube { filterable: true },
                DescriptorBindingType::Sampler { filtering: true },
                DescriptorBindingType::TextureView { filterable: false },
                DescriptorBindingType::Sampler { filtering: false },
            ],
        });

        // buffers
        let scene_buffer =
            rcx.create_uniform_buffer(&SceneData::default().ambient(1.0).ibl_strength(1.0));
        let camera_buffer = rcx.create_uniform_buffer(&Camera3DBufferData::default());

        // Create sampler for irradiance map
        let irradiance_sampler = rcx.create_sampler(SamplerOptions {
            mode_u: TextureMode::ClampToEdge,
            mode_v: TextureMode::ClampToEdge,
            mode_w: TextureMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            compare: None,
        });

        let prefilter_sampler = rcx.create_sampler(SamplerOptions {
            mode_u: TextureMode::ClampToEdge,
            mode_v: TextureMode::ClampToEdge,
            mode_w: TextureMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            compare: None,
        });

        let brdf_lut_sampler = rcx.create_sampler(SamplerOptions {
            mode_u: TextureMode::ClampToEdge,
            mode_v: TextureMode::ClampToEdge,
            mode_w: TextureMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            compare: None,
        });

        // Get the shared light layout from ShadowResource
        let light_layout = ShadowResource::layout(rcx);

        let scene_data = SceneDescriptor {
            layout: scene_layout.clone(),
            scene_buffer,
            camera_data_buffer: camera_buffer,
            irradiance_sampler,
            prefilter_sampler,
            brdf_lut_sampler,
        };

        // Create pipelines
        // Opaque: depth write enabled
        let opaque_depth_mode = DepthMode::Texture(DepthStencilOptions {
            format: TextureFormat::Depth32,
            compare: DepthCompare::Less,
            write_enabled: true,
            depth_bias: None,
        });

        // Blend: depth write disabled (but depth test still enabled)
        let blend_depth_mode = DepthMode::Texture(DepthStencilOptions {
            format: TextureFormat::Depth32,
            compare: DepthCompare::Less,
            write_enabled: false,
            depth_bias: None,
        });

        let opaque_pipeline = rcx.create_pipeline(PipelineCreateInfo {
            label: Some("MainPass_Opaque"),
            layout: rcx.create_pipeline_layout(&[
                scene_layout.clone(),
                material_layout.clone(),
                mesh_layout.clone(),
                light_layout.clone(),
            ]),
            shader: shader.clone(),
            color_formats: &[TextureFormat::RGBA16Float, TextureFormat::RGBA8],
            depth: &opaque_depth_mode,
            cull_mode: CullMode::Back,
            alpha_mode: PipelineAlphaMode::Opaque,
            sample_count: 4,
            use_vertex_buffer: true,
        });

        let blend_pipeline = rcx.create_pipeline(PipelineCreateInfo {
            label: Some("MainPass_Blend"),
            layout: rcx.create_pipeline_layout(&[
                scene_layout.clone(),
                material_layout.clone(),
                mesh_layout.clone(),
                light_layout.clone(),
            ]),
            shader: shader.clone(),
            color_formats: &[TextureFormat::RGBA16Float, TextureFormat::RGBA8],
            depth: &blend_depth_mode,
            cull_mode: CullMode::Back,
            alpha_mode: PipelineAlphaMode::Blend,
            sample_count: 4,
            use_vertex_buffer: true,
        });

        Self {
            pipelines: MainPipelines {
                opaque: opaque_pipeline,
                blend: blend_pipeline,
            },
            scene_data,
            texture_cache: None,
        }
    }
}

impl RenderNode for MainPass {
    fn draw(
        &mut self,
        rcx: &RenderContext,
        graph_ctx: &mut RenderGraphContext,
        game_ctx: &GameContext,
    ) {
        // Refresh textures from graph context if they were cleared during resize
        let targets = self.texture_cache.get_or_insert_with(|| TextureCache {
            msaa_color: graph_ctx
                .get_shared_resource::<Texture>("msaa_color_texture")
                .cloned()
                .unwrap(),
            resolved_color: graph_ctx
                .get_shared_resource::<Texture>("resolved_color_texture")
                .cloned()
                .unwrap(),
            msaa_normal: graph_ctx
                .get_shared_resource::<Texture>("msaa_normal_texture")
                .cloned()
                .unwrap(),
            resolved_normal: graph_ctx
                .get_shared_resource::<Texture>("resolved_normal_texture")
                .cloned()
                .unwrap(),
            msaa_depth: graph_ctx
                .get_shared_resource::<Texture>("main_depth_texture")
                .cloned()
                .unwrap(),
        });

        let scene = &game_ctx.scene;

        let cameras = scene.collect::<Camera3D>();
        let meshes = scene.collect::<Mesh3D>();
        let direct_lights = scene.collect::<DirectionalLight>();
        let point_lights = scene.collect::<PointLight>();
        let environments = scene.collect::<Environment>();

        let Some(camera) = cameras
            .iter()
            .filter(|c| c.read().is_active)
            .max_by_key(|c| c.read().priority)
        else {
            return;
        };

        let camera_frustum = {
            let vp = camera.read().get_vp_matrix(rcx.aspect_ratio());
            Frustum::from_view_proj(&vp)
        };

        let scene_data = &self.scene_data;

        // Get IBL strength from environment (default to 0.0 if there isnt any)
        let ibl_strength = environments
            .first()
            .map(|env| env.read().ibl_strength())
            .unwrap_or(0.0);

        // if no environment then we need to clear the screen since no skybox was rendered
        let clear_color = if environments.is_empty() {
            Some([0.001, 0.001, 0.001, 1.0])
        } else {
            None
        };

        // Update scene buffer with current IBL strength
        let scene_buffer_data = SceneData::default()
            .ambient(0.01)
            .ibl_strength(ibl_strength);
        rcx.write_buffer(&scene_data.scene_buffer, &scene_buffer_data);

        // Get irradiance map from graph context, or use default black cubemap
        let default_textures = rcx.get_default_texture();
        let irradiance_map = graph_ctx
            .get_shared_resource::<TextureCube>("irradiance_cubemap")
            .unwrap_or(&default_textures.irradiance_cubemap);

        let prefilter_map = graph_ctx
            .get_shared_resource::<TextureCube>("prefilter_cubemap")
            .unwrap_or(&default_textures.prefilter_cubemap);

        let brdf_lut_map = graph_ctx
            .get_shared_resource::<Texture>("brdf_lut")
            .unwrap_or(&default_textures.brdf_lut);

        // Build scene descriptor set with irradiance map
        let scene_set = rcx.build_descriptor_set(
            DescriptorSet::builder(&scene_data.layout)
                .uniform(0, &scene_data.scene_buffer)
                .uniform(1, &scene_data.camera_data_buffer)
                .texture_view(2, &irradiance_map.create_view())
                .sampler(3, &scene_data.irradiance_sampler)
                .texture_view(4, &prefilter_map.create_view())
                .sampler(5, &scene_data.prefilter_sampler)
                .texture_view(6, &brdf_lut_map.create_view())
                .sampler(7, &scene_data.brdf_lut_sampler),
        );

        // Get light resources from ShadowResource
        let Some(direct_light_buffer) = (match graph_ctx
            .get_shared_resource::<Buffer<DirectionalLightBuffer>>("direct_light_buffer")
        {
            Some(buf) => Some(buf),
            None => {
                return;
            }
        }) else {
            return;
        };

        let Some(point_light_buffer) = (match graph_ctx
            .get_shared_resource::<Buffer<PointLightBuffer>>("point_light_buffer")
        {
            Some(buf) => Some(buf),
            None => {
                return;
            }
        }) else {
            return;
        };

        let Some(light_set) =
            (match graph_ctx.get_shared_resource::<DescriptorSet>("light_descriptor_set") {
                Some(set) => Some(set),
                None => {
                    return;
                }
            })
        else {
            return;
        };

        // Update light buffers with current scene data
        let direct_light_data = DirectionalLightBuffer::from_lights(
            &direct_lights
                .iter()
                .enumerate()
                .map(|(i, light)| {
                    light
                        .read()
                        .to_buffer_data(&camera.read(), rcx.aspect_ratio(), i)
                })
                .collect::<Vec<_>>(),
        );

        rcx.write_buffer(direct_light_buffer, &direct_light_data);

        let point_light_data = PointLightBuffer::from_lights(
            &point_lights
                .iter()
                .enumerate()
                .map(|(i, light)| light.read().get_buffered_data(i))
                .collect::<Vec<_>>(),
        );

        rcx.write_buffer(point_light_buffer, &point_light_data);

        rcx.write_buffer(
            &scene_data.camera_data_buffer,
            &camera.read().get_buffer_data(rcx.aspect_ratio()),
        );

        let pipelines = &self.pipelines;

        // Sort meshes by alpha mode
        let mut opaque_meshes = Vec::new();
        let mut blend_meshes = Vec::new();

        for mesh in meshes {
            match mesh.read().get_material().alpha_mode() {
                AlphaMode::Opaque | AlphaMode::Mask => opaque_meshes.push(mesh),
                AlphaMode::Blend => blend_meshes.push(mesh),
            }
        }

        rcx.render(
            RenderOptions {
                label: Some("Main Pass"),
                color_targets: &[
                    RenderTarget::MultiSampled {
                        texture: targets.msaa_color.create_view(),
                        resolve: targets.resolved_color.create_view(),
                    },
                    RenderTarget::MultiSampled {
                        texture: targets.msaa_normal.create_view(),
                        resolve: targets.resolved_normal.create_view(),
                    },
                ],
                depth_target: Some(&targets.msaa_depth.create_view()),
                clear_color,
                clear_depth: Some(1.0),
            },
            move |mut fb| {
                fb.bind_descriptor_set(0, &scene_set)
                    .bind_descriptor_set(3, light_set);

                // Render opaque meshes first
                fb.use_pipeline(&pipelines.opaque);
                for mesh in opaque_meshes {
                    let mesh = mesh.read();
                    let Some(material) = mesh.get_material().get_descriptor(rcx, &game_ctx.assets)
                    else {
                        continue;
                    };
                    // cull if outside frustum
                    if !camera_frustum.intersects_aabb(&mesh.world_aabb()) {
                        continue;
                    }
                    fb.bind_vertex_buffer(&mesh.get_vertex_buffer(rcx))
                        .bind_index_buffer(&mesh.get_index_buffer(rcx))
                        .bind_descriptor_set(1, &material)
                        .bind_descriptor_set(2, &mesh.get_descriptor(rcx))
                        .draw_indexed();
                }

                // Render blend meshes after
                fb.use_pipeline(&pipelines.blend);
                for mesh in blend_meshes {
                    let mesh = mesh.read();
                    let Some(material) = mesh.get_material().get_descriptor(rcx, &game_ctx.assets)
                    else {
                        continue;
                    };
                    // cull if outside frustum
                    if !camera_frustum.intersects_aabb(&mesh.world_aabb()) {
                        continue;
                    }
                    fb.bind_vertex_buffer(&mesh.get_vertex_buffer(rcx))
                        .bind_index_buffer(&mesh.get_index_buffer(rcx))
                        .bind_descriptor_set(1, &material)
                        .bind_descriptor_set(2, &mesh.get_descriptor(rcx))
                        .draw_indexed();
                }
            },
        )
        .expect("failed to render");
    }

    fn resize(&mut self, _rcx: &RenderContext, _dimensions: Dimensions) {
        // Textures are recreated by SceneTextures node during resize
        // We just need to clear our cached textures so they get refreshed from graph_ctx in next draw
        self.texture_cache = None;
    }
}
