use std::collections::{HashMap, HashSet};

use bytemuck::{Pod, Zeroable};
use maple_engine::{
    asset::AssetId,
    scene::{NodeHandle, NodeView},
};
use maple_renderer::{
    core::{
        Buffer, CullMode, DescriptorBindingType, DescriptorSet, DescriptorSetLayout,
        DescriptorSetLayoutDescriptor, RenderPipeline, StageFlags, texture::SamplerOptions,
    },
    render_graph::{
        graph::{GraphResource, Stage},
        node::RenderNode,
    },
};
use rayon::{
    iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelIterator},
    slice::ParallelSliceMut,
};

use crate::{
    assets::{
        material::{MaterialAlphaInfo, MaterialPipelineCache},
        mesh::Mesh3D,
    },
    math::AABB,
    nodes::mesh_instance::{Mesh3DUniformBufferData, MeshInstance3D},
    prelude::AlphaMode,
    render_passes::{main_pass::MainPass, shadow_resource::ShadowResource},
};

#[repr(C)]
#[derive(Default, Debug, Pod, Zeroable, Clone, Copy)]
pub(crate) struct AlphaInfoGpu {
    base_alpha_factor: f32,
    alpha_cutoff: f32,
    alpha_mode: u32,
    _padding: [f32; 1],
}

#[derive(Clone)]
pub(crate) struct MeshBundle {
    pub mesh: Mesh3D,
    pub mesh_id: AssetId,
    pub material_id: AssetId,
    pub material_descriptor: DescriptorSet,
    pub shadow_descriptors: DescriptorSet,
    pub pipeline: RenderPipeline,
    pub buffer_data: Mesh3DUniformBufferData,
    pub cull_mode: CullMode,
    pub world_aabb: AABB,
    pub cast_shadow: bool,
}

struct MeshPrepped<'a> {
    is_opaque: bool,
    mesh: &'a NodeView<'a, MeshInstance3D>, // the `mesh` NodeView itself, needed later for transform access
    mesh_instance: Mesh3D,
    mesh_id: AssetId,
    material_id: AssetId,
    pipeline: RenderPipeline,
    material_descriptor: DescriptorSet,
    shadow_descriptor: DescriptorSet,
    cull_mode: CullMode,
    cast_shadow: bool,
}

pub struct CollectMesh {
    shadow_descriptors: HashMap<AssetId, (Buffer<AlphaInfoGpu>, DescriptorSet)>,
    mesh_layout: DescriptorSetLayout,
    scene_layout: DescriptorSetLayout,
    light_layout: DescriptorSetLayout,
    shadow_layout: DescriptorSetLayout,
}

/// mesh bundles collected from the game scene sorted for batching
pub(crate) struct BundledMeshes {
    pub(crate) meshes: Vec<MeshBundle>,
}

impl RenderNode for CollectMesh {
    fn label() -> &'static str
    where
        Self: Sized,
    {
        "Collect Meshes"
    }

    fn stage(&self) -> Stage {
        Stage::PrePass
    }

    fn setup(
        rcx: &maple_renderer::core::RenderContext,
        _graph_ctx: &mut maple_renderer::render_graph::graph::RenderGraphContext,
    ) -> Self
    where
        Self: Sized,
    {
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
        let light_layout = ShadowResource::layout(rcx);
        let shadow_layout = ShadowResource::shadow_layout(rcx);
        Self {
            shadow_descriptors: HashMap::new(),
            mesh_layout,
            scene_layout,
            light_layout,
            shadow_layout,
        }
    }

    fn draw(
        &mut self,
        rcx: &maple_renderer::core::RenderContext,
        _frame: &mut maple_renderer::core::Frame,
        graph_ctx: &mut maple_renderer::render_graph::graph::RenderGraphContext,
        game_ctx: &maple_engine::GameContext,
    ) {
        let meshes = game_ctx.scene.collect::<MeshInstance3D>();

        let mut written_materials: HashSet<AssetId> = HashSet::new();
        let mut material_cache = game_ctx.get_resource_mut::<MaterialPipelineCache>();

        let prepped: Vec<MeshPrepped> = meshes
            .iter()
            .filter_map(|mesh| {
                let (material_id, material_handle, mesh_handle) = {
                    let node = mesh.get_ref();
                    let material = node.material.clone()?;
                    let mesh_h = node.mesh.clone()?;
                    (material.id().clone(), material, mesh_h)
                };
                let mesh_instance = game_ctx.assets.get(&mesh_handle)?.clone();
                let material_instance = game_ctx.assets.get(&material_handle)?;
                let material_descriptor =
                    material_instance.descriptor_set(rcx, &game_ctx.assets)?;

                let is_opaque = matches!(
                    material_instance.alpha_mode(),
                    AlphaMode::Opaque | AlphaMode::Mask
                );
                let cast_shadow = material_instance.casts_shadows();
                let type_id = material_instance.material_key();
                let pipeline_key = material_instance.pipeline_key();

                let pipeline = material_cache
                    .pipelines
                    .entry(type_id)
                    .or_default()
                    .entry(pipeline_key)
                    .or_insert_with(|| {
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
                        material_instance.pipeline(
                            rcx,
                            &MainPass::pass_info(),
                            pipeline_layout,
                            shader,
                        )
                    })
                    .clone();

                let alpha_info =
                    material_instance
                        .alpha_info()
                        .unwrap_or_else(|| MaterialAlphaInfo {
                            alpha_texture: None,
                            base_alpha_factor: 1.0,
                            alpha_cutoff: 0.5,
                        });
                let alpha_info_gpu = AlphaInfoGpu {
                    alpha_mode: material_instance.alpha_mode().into(),
                    base_alpha_factor: alpha_info.base_alpha_factor,
                    alpha_cutoff: alpha_info.alpha_cutoff,
                    _padding: Zeroable::zeroed(),
                };
                let default_alpha_texture = &rcx.get_default_texture().white;
                let alpha_texture = match &alpha_info.alpha_texture {
                    Some(handle) => game_ctx.assets.get(handle)?.clone(),
                    None => default_alpha_texture.clone(),
                };

                let (buffer, shadow_descriptor) = self
                    .shadow_descriptors
                    .entry(material_id)
                    .or_insert_with(|| {
                        let sampler = rcx.device().create_sampler(SamplerOptions {
                            mode_u: maple_renderer::core::texture::TextureMode::Repeat,
                            mode_v: maple_renderer::core::texture::TextureMode::Repeat,
                            mode_w: maple_renderer::core::texture::TextureMode::Repeat,
                            mag_filter: maple_renderer::core::texture::FilterMode::Linear,
                            min_filter: maple_renderer::core::texture::FilterMode::Linear,
                            compare: None,
                        });
                        let buffer = rcx.device().create_uniform_buffer(&alpha_info_gpu);
                        let descriptor = rcx.device().build_descriptor_set(
                            &DescriptorSet::builder(&self.shadow_layout)
                                .uniform(0, &buffer)
                                .texture_view(1, &alpha_texture.create_view())
                                .sampler(2, &sampler),
                        );
                        (buffer, descriptor)
                    });

                if !written_materials.contains(material_handle.id()) {
                    rcx.queue().write_buffer(buffer, &alpha_info_gpu);
                    written_materials.insert(material_handle.id().clone());
                }

                Some(MeshPrepped {
                    is_opaque,
                    mesh,
                    mesh_instance,
                    mesh_id: mesh_handle.id().clone(),
                    material_id: material_handle.id().clone(),
                    pipeline,
                    material_descriptor,
                    shadow_descriptor: shadow_descriptor.clone(),
                    cull_mode: material_instance.cull_mode(),
                    cast_shadow,
                })
            })
            .collect();

        let (mut opaque_bundles, mut transparent_bundles): (Vec<_>, Vec<_>) = prepped
            .into_par_iter()
            .map(|p| {
                let world_space = *p.mesh.get_ref().transform.world_space();
                let world_aabb = p.mesh_instance.world_aabb(world_space);
                let buffer_data = Mesh3DUniformBufferData {
                    model: world_space.matrix.to_cols_array_2d(),
                    normal_matrix: world_space.matrix.inverse().transpose().to_cols_array_2d(),
                };

                let bundle = MeshBundle {
                    mesh: p.mesh_instance,
                    mesh_id: p.mesh_id,
                    material_descriptor: p.material_descriptor,
                    shadow_descriptors: p.shadow_descriptor,
                    material_id: p.material_id,
                    pipeline: p.pipeline,
                    world_aabb,
                    cull_mode: p.cull_mode,
                    buffer_data,
                    cast_shadow: p.cast_shadow,
                };
                (p.is_opaque, bundle)
            })
            .partition_map(|(is_opaque, bundle)| {
                if is_opaque {
                    rayon::iter::Either::Left(bundle)
                } else {
                    rayon::iter::Either::Right(bundle)
                }
            });

        opaque_bundles.par_sort_unstable_by_key(|bundle| {
            (
                bundle.pipeline.id.clone(),
                bundle.material_id.clone(),
                bundle.mesh_id.clone(),
            )
        });

        transparent_bundles.par_sort_unstable_by_key(|bundle| {
            (
                bundle.pipeline.id.clone(),
                bundle.material_id.clone(),
                bundle.mesh_id.clone(),
            )
        });

        opaque_bundles.append(&mut transparent_bundles);
        let mesh_bundles = BundledMeshes {
            meshes: opaque_bundles,
        };

        graph_ctx.add_shared_resource(mesh_bundles);
    }
}
