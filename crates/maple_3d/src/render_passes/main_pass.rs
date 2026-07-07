use std::{sync::Arc, time::Instant};

use bytemuck::{Pod, Zeroable};
use maple_engine::{
    GameContext,
    asset::{AssetId, AssetState},
    prelude::node_transform::WorldTransform,
};
use maple_renderer::{
    core::{
        Buffer, DescriptorBindingType, DescriptorSet, DescriptorSetBuilder,
        DescriptorSetLayoutDescriptor, Frame, RenderContext, StageFlags,
        context::RenderOptions,
        descriptor_set::DescriptorSetLayout,
        pipeline::RenderPipeline,
        texture::{
            FilterMode, Sampler, SamplerOptions, Texture, TextureCube, TextureFormat, TextureMode,
        },
    },
    render_graph::{
        graph::RenderGraphContext,
        node::{RenderNode, RenderTarget},
    },
    types::Dimensions,
};

use crate::{
    assets::mesh::Mesh3D,
    math::Frustum,
    nodes::{
        camera::{Camera3D, Camera3DBufferData},
        directional_light::{DirectionalLight, DirectionalLightBuffer},
        environment::Environment,
        mesh_instance::{Mesh3DUniformBufferData, MeshInstance3D},
        point_light::{PointLight, PointLightBuffer},
    },
    prelude::{AlphaMode, Material, MaterialPipelineCache, PassInfo},
    render_passes::shadow_resource::ShadowResource,
};

pub const MAX_MESH: usize = 1024;

struct SceneDescriptor {
    pub layout: DescriptorSetLayout,
    pub scene_buffer: Buffer<SceneData>,
    pub camera_data_buffer: Buffer<Camera3DBufferData>,
    pub irradiance_sampler: Sampler,
    pub prefilter_sampler: Sampler,
    pub brdf_lut_sampler: Sampler,
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

struct PipelineBatch {
    material_batches: Vec<MaterialBatch>,
    pipeline: RenderPipeline,
    pipeline_id: AssetId,
}

struct MaterialBatch {
    mesh_batches: Vec<MeshBatch>,
    material: Arc<Material>,
    material_descriptor: DescriptorSet,
    material_id: AssetId,
}

struct MeshBatch {
    mesh: Arc<Mesh3D>,
    mesh_id: AssetId,
    start: u32,
    end: u32,
}

struct MeshBundle {
    mesh: Arc<Mesh3D>,
    mesh_id: AssetId,
    material: Arc<Material>,
    material_descriptor: DescriptorSet,
    material_id: AssetId,
    pipeline: RenderPipeline,
    world_transform: WorldTransform,
    mesh_index: u32,
}

struct TextureCache {
    msaa_color: Texture,
    resolved_color: Texture,
    msaa_normal: Texture,
    resolved_normal: Texture,
    msaa_depth: Texture,
}

pub struct MainPass {
    scene_data: SceneDescriptor,
    // Render targets cached so we dont need to fetch from graph every frame (maybe this is useless)
    texture_cache: Option<TextureCache>,
    pass_info: PassInfo,
    scene_layout: DescriptorSetLayout,
    mesh_layout: DescriptorSetLayout,
    light_layout: DescriptorSetLayout,
    mesh_buffer: Buffer<[Mesh3DUniformBufferData]>,
    mesh_descriptor: DescriptorSet,
}

impl MainPass {
    pub fn batch_meshes(
        meshes: &Vec<MeshBundle>,
    ) -> (Vec<PipelineBatch>, Vec<Mesh3DUniformBufferData>) {
        let mut order: Vec<usize> = (0..meshes.len()).collect();
        order.sort_unstable_by_key(|&i| {
            let b = &meshes[i];
            (
                b.pipeline.id.clone(),
                b.material_id.clone(),
                b.mesh_id.clone(),
            )
        });

        let mut batch_pipelines: Vec<PipelineBatch> = Vec::new();
        let mut mesh_buffer: Vec<Mesh3DUniformBufferData> = Vec::new();

        for i in order {
            let bundle = &meshes[i];
            let pipeline_id = bundle.pipeline.id.clone();
            let material_id = bundle.material_id.clone();
            let mesh_id = bundle.mesh_id.clone();

            let instance_index = mesh_buffer.len() as u32;
            mesh_buffer.push(Mesh3DUniformBufferData {
                model: bundle.world_transform.matrix.to_cols_array_2d(),
                normal_matrix: bundle
                    .world_transform
                    .matrix
                    .inverse()
                    .transpose()
                    .to_cols_array_2d(),
            });

            if batch_pipelines.last().map(|b| &b.pipeline_id) != Some(&pipeline_id) {
                batch_pipelines.push(PipelineBatch {
                    material_batches: Vec::new(),
                    pipeline: bundle.pipeline.clone(),
                    pipeline_id,
                })
            }
            let bp = batch_pipelines.last_mut().unwrap();

            if bp.material_batches.last().map(|b| &b.material_id) != Some(&material_id) {
                bp.material_batches.push(MaterialBatch {
                    mesh_batches: Vec::new(),
                    material: bundle.material.clone(),
                    material_descriptor: bundle.material_descriptor.clone(),
                    material_id,
                })
            }
            let bm = bp.material_batches.last_mut().unwrap();

            if let Some(last) = bm.mesh_batches.last_mut() {
                if last.mesh_id == mesh_id && last.end == instance_index {
                    last.end = instance_index + 1;
                    continue;
                }
            }
            bm.mesh_batches.push(MeshBatch {
                mesh: bundle.mesh.clone(),
                mesh_id,
                start: instance_index,
                end: instance_index + 1,
            })
        }

        (batch_pipelines, mesh_buffer)
    }
}

impl RenderNode for MainPass {
    fn setup(rcx: &RenderContext, _gcx: &mut RenderGraphContext) -> Self {
        // layouts
        let mesh_layout = rcx.get_or_create_layout(DescriptorSetLayoutDescriptor {
            label: Some("Mesh"),
            visibility: StageFlags::VERTEX,
            layout: &[
                DescriptorBindingType::Storage {
                    read_only: true,
                    has_dynamic_offset: false,
                    min_size: None,
                }, // transforms
            ],
        });
        let mesh_buffer = rcx
            .device()
            .create_sized_storage_buffer(size_of::<Mesh3DUniformBufferData>() * MAX_MESH);
        let mesh_descriptor = rcx
            .device()
            .build_descriptor_set(&DescriptorSet::builder(&mesh_layout).storage(0, &mesh_buffer));

        let scene_layout =
            rcx.device()
                .create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
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
        let scene_buffer = rcx
            .device()
            .create_uniform_buffer(&SceneData::default().ambient(1.0).ibl_strength(1.0));
        let camera_buffer = rcx
            .device()
            .create_uniform_buffer(&Camera3DBufferData::default());

        // Create sampler for irradiance map
        let irradiance_sampler = rcx.device().create_sampler(SamplerOptions {
            mode_u: TextureMode::ClampToEdge,
            mode_v: TextureMode::ClampToEdge,
            mode_w: TextureMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            compare: None,
        });

        let prefilter_sampler = rcx.device().create_sampler(SamplerOptions {
            mode_u: TextureMode::ClampToEdge,
            mode_v: TextureMode::ClampToEdge,
            mode_w: TextureMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            compare: None,
        });

        let brdf_lut_sampler = rcx.device().create_sampler(SamplerOptions {
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

        Self {
            scene_data,
            texture_cache: None,
            pass_info: PassInfo {
                color_formats: vec![TextureFormat::RGBA16Float, TextureFormat::RGBA8],
                sample_count: 4,
            },
            scene_layout,
            mesh_layout,
            light_layout,
            mesh_buffer,
            mesh_descriptor,
        }
    }
    fn draw(
        &mut self,
        rcx: &RenderContext,
        frame: &mut Frame,
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
        let meshes_instances = scene.collect::<MeshInstance3D>();
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
            Some([0.01, 0.01, 0.01, 1.0])
        } else {
            None
        };

        // Update scene buffer with current IBL strength
        let scene_buffer_data = SceneData::default()
            .ambient(0.01)
            .ibl_strength(ibl_strength);
        rcx.queue()
            .write_buffer(&scene_data.scene_buffer, &scene_buffer_data);

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
        let scene_set = rcx.device().build_descriptor_set(
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

        rcx.queue()
            .write_buffer(direct_light_buffer, &direct_light_data);

        let point_light_data = PointLightBuffer::from_lights(
            &point_lights
                .iter()
                .enumerate()
                .map(|(i, light)| light.read().get_buffered_data(i))
                .collect::<Vec<_>>(),
        );

        rcx.queue()
            .write_buffer(point_light_buffer, &point_light_data);

        rcx.queue().write_buffer(
            &scene_data.camera_data_buffer,
            &camera.read().get_buffer_data(rcx.aspect_ratio()),
        );

        let mut material_cache = game_ctx.get_resource_mut::<MaterialPipelineCache>();
        let mut opaque_meshes = Vec::new();
        let mut blend_meshes = Vec::new();

        for (mesh_idx, mesh) in meshes_instances.iter().enumerate() {
            let (material_handle, mesh_handle) = {
                let node = mesh.read();
                let Some(material) = node.material.clone() else {
                    continue;
                };
                let Some(mesh) = node.mesh.clone() else {
                    continue;
                };
                (material, mesh)
            };
            let AssetState::Loaded(mesh_instance) = game_ctx.assets.get(&mesh_handle) else {
                continue;
            };

            if !camera_frustum
                .intersects_aabb(&mesh_instance.world_aabb(*mesh.read().transform.world_space()))
            {
                continue;
            }

            let AssetState::Loaded(material_instance) = game_ctx.assets.get(&material_handle)
            else {
                continue;
            };

            let Some(material_descriptor) = material_instance.descriptor_set(rcx, &game_ctx.assets)
            else {
                continue;
            };

            let is_opaque = matches!(
                material_instance.alpha_mode(),
                AlphaMode::Opaque | AlphaMode::Mask
            );
            let key = material_instance.material_key();
            let cache = if is_opaque {
                &mut material_cache.opaque
            } else {
                &mut material_cache.transparent
            };

            let pipeline = cache.entry(key).or_insert_with(|| {
                let shader = maple_renderer::core::GraphicsShader {
                    vertex: rcx
                        .device()
                        .compile_shader(material_instance.vertex_shader())
                        .expect("material vertex shader compile"),
                    fragment: rcx
                        .device()
                        .compile_shader(material_instance.fragment_shader())
                        .expect("material fragment shader compile"),
                };
                let material_layout = material_instance.layout(rcx);
                let pipeline_layout = rcx.device().create_render_pipeline_layout(&[
                    self.scene_layout.clone(),
                    self.mesh_layout.clone(),
                    self.light_layout.clone(),
                    material_layout,
                ]);
                material_instance.pipeline(rcx, &self.pass_info, pipeline_layout, shader)
            });

            let bundle = MeshBundle {
                mesh: mesh_instance.clone(),
                mesh_id: mesh_handle.id,
                material: material_instance.clone(),
                material_descriptor,
                material_id: material_handle.id,
                pipeline: pipeline.clone(),
                world_transform: *mesh.read().transform.world_space(),
                mesh_index: mesh_idx as u32,
            };

            if is_opaque {
                opaque_meshes.push(bundle);
            } else {
                blend_meshes.push(bundle);
            }
        }

        let (mut batches, mut buffer_data) = Self::batch_meshes(&opaque_meshes);
        let (mut blend_batches, mut blend_buffer_data) = Self::batch_meshes(&blend_meshes);
        batches.append(&mut blend_batches);
        buffer_data.append(&mut blend_buffer_data);

        rcx.queue()
            .write_buffer_slice(&self.mesh_buffer, &buffer_data);

        frame
            .render(
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
                        .bind_descriptor_set(1, &self.mesh_descriptor)
                        .bind_descriptor_set(2, light_set);

                    for pipeline_batch in batches {
                        fb.use_pipeline(&pipeline_batch.pipeline);

                        for material_batch in pipeline_batch.material_batches {
                            fb.bind_descriptor_set(3, &material_batch.material_descriptor);

                            for mesh_batch in material_batch.mesh_batches {
                                fb.bind_vertex_buffer(&mesh_batch.mesh.get_vertex_buffer())
                                    .bind_index_buffer(&mesh_batch.mesh.get_index_buffer())
                                    .draw_indexed(mesh_batch.start..mesh_batch.end);
                            }
                        }
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
