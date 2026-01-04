use bytemuck::{Pod, Zeroable};
use maple_engine::{Scene, utils::Debug};
use maple_renderer::{
    core::{
        Buffer, CullMode, DepthCompare, DepthStencilOptions, DescriptorBindingType, DescriptorSet,
        DescriptorSetLayoutDescriptor, GraphicsShader, RenderContext, StageFlags,
        context::RenderOptions,
        descriptor_set::DescriptorSetLayout,
        pipeline::{AlphaMode as PipelineAlphaMode, PipelineCreateInfo, RenderPipeline},
        texture::{
            FilterMode, Sampler, SamplerOptions, Texture, TextureCube, TextureFormat, TextureMode,
        },
    },
    render_graph::{
        graph::{NodeLabel, RenderGraphContext},
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
    nodes::{
        camera::{Camera3D, Camera3DBufferData},
        directional_light::{DirectionalLight, DirectionalLightBuffer},
        environment::Environment,
        mesh::Mesh3D,
        point_light::{PointLight, PointLightBuffer},
    },
    render_passes::shadow_resource::ShadowResource,
};

pub struct Main;
impl NodeLabel for Main {}

#[derive(Default)]
pub struct MainPass {
    scene_data: Option<SceneDescriptor>,
    _normal_texture: Option<Texture>,
    pipelines: Option<MainPipelines>,
    shader: Option<GraphicsShader>,
    // Render targets
    msaa_color: Option<Texture>,
    resolved_color: Option<Texture>,
    msaa_normal: Option<Texture>,
    resolved_normal: Option<Texture>,
    msaa_depth: Option<Texture>,
}

impl RenderNode for MainPass {
    fn setup(&mut self, render_ctx: &RenderContext, _graph_ctx: &mut RenderGraphContext) {
        // shader
        let shader = render_ctx.create_shader_pair(maple_renderer::core::ShaderPair::Wgsl {
            vert: include_str!("../../res/shaders/default/default.vert.wgsl"),
            frag: include_str!("../../res/shaders/default/default.frag.wgsl"),
        });

        // layouts
        let material_layout = MaterialProperties::layout(render_ctx).clone();
        let mesh_layout = Mesh3D::layout(render_ctx).clone();
        let scene_layout = render_ctx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
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
            render_ctx.create_uniform_buffer(&SceneData::default().ambient(1.0).ibl_strength(1.0));
        let camera_buffer = render_ctx.create_uniform_buffer(&Camera3DBufferData::default());

        // Create sampler for irradiance map
        let irradiance_sampler = render_ctx.create_sampler(SamplerOptions {
            mode_u: TextureMode::ClampToEdge,
            mode_v: TextureMode::ClampToEdge,
            mode_w: TextureMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            compare: None,
        });

        let prefilter_sampler = render_ctx.create_sampler(SamplerOptions {
            mode_u: TextureMode::ClampToEdge,
            mode_v: TextureMode::ClampToEdge,
            mode_w: TextureMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            compare: None,
        });

        let brdf_lut_sampler = render_ctx.create_sampler(SamplerOptions {
            mode_u: TextureMode::ClampToEdge,
            mode_v: TextureMode::ClampToEdge,
            mode_w: TextureMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            compare: None,
        });

        // Get the shared light layout from ShadowResource
        let light_layout = ShadowResource::layout(render_ctx);

        self.scene_data = Some(SceneDescriptor {
            layout: scene_layout.clone(),
            scene_buffer,
            camera_data_buffer: camera_buffer,
            irradiance_sampler,
            prefilter_sampler,
            brdf_lut_sampler,
        });

        let surface_format = render_ctx.surface_format();

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

        let opaque_pipeline = render_ctx.create_pipeline(PipelineCreateInfo {
            label: Some("MainPass_Opaque"),
            layout: render_ctx.create_pipeline_layout(&[
                scene_layout.clone(),
                material_layout.clone(),
                mesh_layout.clone(),
                light_layout.clone(),
            ]),
            shader: shader.clone(),
            color_formats: &[surface_format, TextureFormat::RGBA8],
            depth: &opaque_depth_mode,
            cull_mode: CullMode::Back, // Temporarily disable culling to test
            alpha_mode: PipelineAlphaMode::Opaque,
            sample_count: 4,
            use_vertex_buffer: true,
        });

        let blend_pipeline = render_ctx.create_pipeline(PipelineCreateInfo {
            label: Some("MainPass_Blend"),
            layout: render_ctx.create_pipeline_layout(&[
                scene_layout.clone(),
                material_layout.clone(),
                mesh_layout.clone(),
                light_layout.clone(),
            ]),
            shader: shader.clone(),
            color_formats: &[surface_format, TextureFormat::RGBA8],
            depth: &blend_depth_mode,
            cull_mode: CullMode::Back,
            alpha_mode: PipelineAlphaMode::Blend,
            sample_count: 4,
            use_vertex_buffer: true,
        });

        self.pipelines = Some(MainPipelines {
            opaque: opaque_pipeline,
            blend: blend_pipeline,
        });

        self.shader = Some(shader);
    }

    fn draw(
        &mut self,
        renderer_ctx: &RenderContext,
        graph_ctx: &mut RenderGraphContext,
        scene: &Scene,
    ) {
        // Refresh textures from graph context if they were cleared during resize
        if self.msaa_color.is_none() {
            self.msaa_color = graph_ctx
                .get_shared_resource::<Texture>("msaa_color_texture")
                .cloned();
            self.resolved_color = graph_ctx
                .get_shared_resource::<Texture>("resolved_color_texture")
                .cloned();
            self.msaa_normal = graph_ctx
                .get_shared_resource::<Texture>("msaa_normal_texture")
                .cloned();
            self.resolved_normal = graph_ctx
                .get_shared_resource::<Texture>("resolved_normal_texture")
                .cloned();
            self.msaa_depth = graph_ctx
                .get_shared_resource::<Texture>("main_depth_texture")
                .cloned();
        }

        let cameras = scene.collect_items::<Camera3D>();
        let meshes = scene.collect_items::<Mesh3D>();
        let direct_lights = scene.collect_items::<DirectionalLight>();
        let point_lights = scene.collect_items::<PointLight>();
        let environments = scene.collect_items::<Environment>();

        let Some(camera) = cameras
            .iter()
            .filter(|c| c.is_active)
            .max_by_key(|c| c.priority)
        else {
            Debug::print_once("no active camera in scene");
            return;
        };

        let Some(scene_data) = &self.scene_data else {
            return;
        };

        // Get IBL strength from environment (default to 0.0 if there isnt any)
        let ibl_strength = environments
            .first()
            .map(|env| env.ibl_strength())
            .unwrap_or(0.0);

        // if no environment then we need to clear the screen since no skybox was rendered
        let clear_color = if environments.is_empty() {
            Some([0.0, 0.0, 0.0, 1.0])
        } else {
            None
        };

        // Update scene buffer with current IBL strength
        let scene_buffer_data = SceneData::default().ambient(0.0).ibl_strength(ibl_strength);
        renderer_ctx.write_buffer(&scene_data.scene_buffer, &scene_buffer_data);

        // Get irradiance map from graph context, or use default black cubemap
        let default_textures = renderer_ctx.get_default_texture();
        let irradiance_map = graph_ctx
            .get_shared_resource::<TextureCube>("irradiance_cubemap")
            .unwrap_or_else(|| {
                log::debug!("No irradiance cubemap found, using default black cubemap");
                &default_textures.irradiance_cubemap
            });

        let prefilter_map = graph_ctx
            .get_shared_resource::<TextureCube>("prefilter_cubemap")
            .unwrap_or_else(|| {
                log::debug!("No prefilter cubemap found, using default black cubemap");
                &default_textures.prefilter_cubemap
            });

        let brdf_lut_map = graph_ctx
            .get_shared_resource::<Texture>("brdf_lut")
            .unwrap_or_else(|| {
                log::debug!("No BRDF LUT found, using default texture");
                &default_textures.brdf_lut
            });

        // Build scene descriptor set with irradiance map
        let scene_set = renderer_ctx.build_descriptor_set(
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
                Debug::print_once("Missing direct light buffer in graph context");
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
                Debug::print_once("Missing point light buffer in graph context");
                return;
            }
        }) else {
            return;
        };

        let Some(light_set) =
            (match graph_ctx.get_shared_resource::<DescriptorSet>("light_descriptor_set") {
                Some(set) => Some(set),
                None => {
                    Debug::print_once("Missing light descriptor set in graph context");
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
                .map(|(i, light)| light.to_buffer_data(camera, renderer_ctx.aspect_ratio(), i))
                .collect::<Vec<_>>(),
        );

        renderer_ctx.write_buffer(direct_light_buffer, &direct_light_data);

        let point_light_data = PointLightBuffer::from_lights(
            &point_lights
                .iter()
                .enumerate()
                .map(|(i, light)| light.get_buffered_data(i))
                .collect::<Vec<_>>(),
        );

        renderer_ctx.write_buffer(point_light_buffer, &point_light_data);

        renderer_ctx.write_buffer(
            &scene_data.camera_data_buffer,
            &camera.get_buffer_data(renderer_ctx.aspect_ratio()),
        );

        let Some(pipelines) = &self.pipelines else {
            return;
        };

        // Sort meshes by alpha mode
        let mut opaque_meshes = Vec::new();
        let mut blend_meshes = Vec::new();

        for mesh in meshes {
            match mesh.get_material().alpha_mode() {
                AlphaMode::Opaque | AlphaMode::Mask => opaque_meshes.push(mesh),
                AlphaMode::Blend => blend_meshes.push(mesh),
            }
        }

        // Get render targets
        let msaa_color = self
            .msaa_color
            .as_ref()
            .expect("msaa_color not initialized")
            .create_view();
        let resolved_color = self
            .resolved_color
            .as_ref()
            .expect("resolved_color not initialized")
            .create_view();
        let msaa_normal = self
            .msaa_normal
            .as_ref()
            .expect("msaa_normal not initialized")
            .create_view();
        let resolved_normal = self
            .resolved_normal
            .as_ref()
            .expect("resolved_normal not initialized")
            .create_view();
        let msaa_depth = self
            .msaa_depth
            .as_ref()
            .expect("msaa_depth not initialized")
            .create_view();

        renderer_ctx
            .render(
                RenderOptions {
                    label: Some("Main Pass"),
                    color_targets: &[
                        RenderTarget::MultiSampled {
                            texture: msaa_color,
                            resolve: resolved_color,
                        },
                        RenderTarget::MultiSampled {
                            texture: msaa_normal,
                            resolve: resolved_normal,
                        },
                    ],
                    depth_target: Some(&msaa_depth),
                    clear_color,
                },
                move |mut fb| {
                    fb.bind_descriptor_set(0, &scene_set)
                        .bind_descriptor_set(3, light_set);

                    // Render opaque meshes first
                    fb.use_pipeline(&pipelines.opaque);
                    for mesh in opaque_meshes {
                        fb.bind_vertex_buffer(&mesh.get_vertex_buffer(renderer_ctx))
                            .bind_index_buffer(&mesh.get_index_buffer(renderer_ctx))
                            .bind_descriptor_set(
                                1,
                                &mesh.get_material().get_descriptor(renderer_ctx),
                            )
                            .bind_descriptor_set(2, &mesh.get_descriptor(renderer_ctx))
                            .draw_indexed();
                    }

                    // Render blend meshes after
                    fb.use_pipeline(&pipelines.blend);
                    for mesh in blend_meshes {
                        fb.bind_vertex_buffer(&mesh.get_vertex_buffer(renderer_ctx))
                            .bind_index_buffer(&mesh.get_index_buffer(renderer_ctx))
                            .bind_descriptor_set(
                                1,
                                &mesh.get_material().get_descriptor(renderer_ctx),
                            )
                            .bind_descriptor_set(2, &mesh.get_descriptor(renderer_ctx))
                            .draw_indexed();
                    }
                },
            )
            .expect("failed to render");
    }

    fn resize(&mut self, _render_ctx: &RenderContext, _dimensions: [u32; 2]) {
        // Textures are recreated by SceneTextures node during resize
        // We just need to clear our cached textures so they get refreshed from graph_ctx in next draw
        self.msaa_color = None;
        self.resolved_color = None;
        self.msaa_normal = None;
        self.resolved_normal = None;
        self.msaa_depth = None;

        // Note: Pipelines don't need to be recreated as they reference depth by value during draw
    }
}
