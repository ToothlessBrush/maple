use std::{collections::HashMap, sync::Arc};

use maple_engine::{
    asset::{AssetId, AssetState},
    prelude::node_transform::WorldTransform,
    scene::NodeId,
};
use maple_renderer::{
    core::{
        DescriptorBindingType, DescriptorSet, DescriptorSetLayout, DescriptorSetLayoutDescriptor,
        RenderPipeline, StageFlags,
    },
    render_graph::{graph::GraphResource, node::RenderNode},
};

use crate::{
    assets::mesh::Mesh3D,
    math::AABB,
    nodes::mesh_instance::{Mesh3DUniformBufferData, MeshInstance3D},
    prelude::{AlphaMode, Material, MaterialPipelineCache},
    render_passes::{main_pass::MainPass, shadow_resource::ShadowResource},
};

#[derive(Clone)]
pub(crate) struct MeshBundle {
    pub mesh: Mesh3D,
    pub mesh_id: AssetId,
    pub material_id: AssetId,
    pub material_descriptor: DescriptorSet,
    pub pipeline: RenderPipeline,
    pub buffer_data: Mesh3DUniformBufferData,
    pub alpha_mode: AlphaMode,
    pub world_aabb: AABB,
    pub cast_shadow: bool,
}

pub struct CollectMesh {
    mesh_cache: HashMap<NodeId, MeshBundle>,
    mesh_layout: DescriptorSetLayout,
    scene_layout: DescriptorSetLayout,
    light_layout: DescriptorSetLayout,
}

/// mesh bundles collected from the game scene sorted for batching
pub(crate) struct BundledMeshes {
    pub(crate) meshes: Vec<MeshBundle>,
}

impl GraphResource for BundledMeshes {}

impl RenderNode for CollectMesh {
    fn setup(
        rcx: &maple_renderer::core::RenderContext,
        graph_ctx: &mut maple_renderer::render_graph::graph::RenderGraphContext,
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
        Self {
            mesh_cache: HashMap::new(),
            mesh_layout,
            scene_layout,
            light_layout,
        }
    }

    fn draw(
        &mut self,
        rcx: &maple_renderer::core::RenderContext,
        frame: &mut maple_renderer::core::Frame,
        graph_ctx: &mut maple_renderer::render_graph::graph::RenderGraphContext,
        game_ctx: &maple_engine::GameContext,
    ) {
        let meshes = game_ctx.scene.collect::<MeshInstance3D>();
        let mut material_cache = game_ctx.get_resource_mut::<MaterialPipelineCache>();

        let mut opaque_bundles: Vec<MeshBundle> = Vec::new();
        let mut transparent_bundles: Vec<MeshBundle> = Vec::new();

        for mesh in meshes {
            if let Some(entry) = self.mesh_cache.get_mut(&mesh.id()) {
                let mesh_handle = {
                    let node = mesh.read();
                    let Some(mesh) = node.mesh.clone() else {
                        continue;
                    };
                    mesh
                };
                let Some(mesh_instance) = game_ctx.assets.get(&mesh_handle) else {
                    continue;
                };
                entry.world_aabb = mesh_instance.world_aabb(*mesh.read().transform.world_space());
                entry.buffer_data = Mesh3DUniformBufferData {
                    model: mesh
                        .read()
                        .transform
                        .world_space()
                        .matrix
                        .to_cols_array_2d(),
                    normal_matrix: mesh
                        .read()
                        .transform
                        .world_space()
                        .matrix
                        .inverse()
                        .transpose()
                        .to_cols_array_2d(),
                };

                match entry.alpha_mode {
                    AlphaMode::Opaque | AlphaMode::Mask => opaque_bundles.push(entry.clone()),
                    AlphaMode::Blend => transparent_bundles.push(entry.clone()),
                }
            } else {
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
                let Some(mesh_instance) = game_ctx.assets.get(&mesh_handle) else {
                    continue;
                };
                let world_aabb = mesh_instance.world_aabb(*mesh.read().transform.world_space());
                let Some(material_instance) = game_ctx.assets.get(&material_handle) else {
                    continue;
                };

                let Some(material_descriptor) =
                    material_instance.descriptor_set(rcx, &game_ctx.assets)
                else {
                    continue;
                };

                let is_opaque = matches!(
                    material_instance.alpha_mode(),
                    AlphaMode::Opaque | AlphaMode::Mask
                );
                let cast_shadow = material_instance.casts_shadows();
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
                    material_instance.pipeline(rcx, &MainPass::pass_info(), pipeline_layout, shader)
                });

                let buffer_data = Mesh3DUniformBufferData {
                    model: mesh
                        .read()
                        .transform
                        .world_space()
                        .matrix
                        .to_cols_array_2d(),
                    normal_matrix: mesh
                        .read()
                        .transform
                        .world_space()
                        .matrix
                        .inverse()
                        .transpose()
                        .to_cols_array_2d(),
                };

                let bundle = MeshBundle {
                    mesh: mesh_instance.clone(),
                    mesh_id: mesh_handle.id,
                    material_descriptor,
                    material_id: material_handle.id,
                    pipeline: pipeline.clone(),
                    world_aabb,
                    alpha_mode: material_instance.alpha_mode(),
                    buffer_data,
                    cast_shadow,
                };
                if is_opaque {
                    opaque_bundles.push(bundle);
                } else {
                    transparent_bundles.push(bundle);
                }
            }
        }

        opaque_bundles.sort_unstable_by_key(|bundle| {
            (
                bundle.pipeline.id.clone(),
                bundle.material_id.clone(),
                bundle.mesh_id.clone(),
            )
        });

        transparent_bundles.sort_unstable_by_key(|bundle| {
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

        graph_ctx.add_shared_resource("mesh_bundles", mesh_bundles);
    }
}
