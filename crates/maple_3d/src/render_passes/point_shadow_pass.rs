use std::collections::HashMap;

use bytemuck::{Pod, Zeroable};
use maple_engine::GameContext;
use maple_renderer::{
    core::{
        Buffer, CullMode, DepthBias, DepthCompare, DepthStencilOptions, DescriptorSetLayout, Frame,
        GraphicsShader, RenderContext, StageFlags,
        context::RenderOptions,
        descriptor_set::{DescriptorBindingType, DescriptorSet, DescriptorSetLayoutDescriptor},
        pipeline::{AlphaMode, PipelineCreateInfo, RenderPipeline},
        texture::{CubeFace, TextureCubeArray, TextureFormat},
    },
    render_graph::{
        graph::{RenderGraphContext, Stage},
        node::{DepthMode, RenderNode},
    },
    types::vertex::VertexLayout,
};

use crate::{
    math::{Frustum, Vertex},
    nodes::{
        mesh_instance::{Mesh3DUniformBufferData, MeshInstance3D},
        point_light::{PointLight, PointLightBuffer},
    },
    render_passes::{
        collect_mesh,
        main_pass::MAX_MESH,
        shadow_resource::{self, ShadowResource},
    },
};

/// Uniform buffer for point light shadow data
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct PointLightShadowUniform {
    view_projection: [[f32; 4]; 4], // 64 bytes
    light_pos: [f32; 4],            // 16 bytes
    far_plane: f32,                 // 4 bytes
    _padding: [u8; 172],            // 172 bytes (total: 256 bytes to match WGSL alignment)
}

/// Point shadow pass renders depth from point light perspectives to cube maps
///
/// This pass renders each point light's shadow cube map by:
/// 1. Getting the light's 6 view-projection matrices (one per cube face)
/// 2. Rendering all meshes from each face's perspective
/// 3. Storing depth values for shadow sampling in the main pass
pub struct PointShadowPass {
    // Buffer for light shadow data
    light_buffer: Buffer<[PointLightShadowUniform]>,

    // Descriptor set for light data
    light_descriptor: DescriptorSet,

    // Render pipeline
    pipeline: HashMap<CullMode, RenderPipeline>,

    mesh_buffers: HashMap<u32, Buffer<[Mesh3DUniformBufferData]>>,
    mesh_layout: DescriptorSetLayout,
    mesh_descriptors: HashMap<u32, DescriptorSet>,
}

impl PointShadowPass {}

impl RenderNode for PointShadowPass {
    fn label() -> &'static str
    where
        Self: Sized,
    {
        "Point Shadow"
    }

    fn stage(&self) -> Stage {
        Stage::Shadow
    }

    fn setup(rcx: &RenderContext, _gcx: &mut RenderGraphContext) -> Self {
        let shader = GraphicsShader {
            vertex: rcx
                .device()
                .compile_shader(include_str!("./point_shadow.vert.wgsl").into())
                .expect("compiled vertex shader"),
            fragment: rcx
                .device()
                .compile_shader(include_str!("./point_shadow.frag.wgsl").into())
                .expect("compiled fragment shader"),
        };

        // Create descriptor set layout for light data
        let light_layout =
            rcx.device()
                .create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
                    label: Some("PointShadow_Light"),
                    visibility: StageFlags::VERTEX | StageFlags::FRAGMENT,
                    layout: &[DescriptorBindingType::Storage {
                        read_only: true,
                        has_dynamic_offset: true,
                        min_size: Some(size_of::<PointLightShadowUniform>()),
                    }], // Binding 0: light data
                });

        // Create buffer for light data
        let light_buffer = rcx.device().create_sized_storage_buffer(
            size_of::<PointLightShadowUniform>() * shadow_resource::POINT_SHADOW_SIZE as usize,
        );

        // Build descriptor set
        let light_descriptor = rcx.device().build_descriptor_set(
            DescriptorSet::builder(&light_layout).storage_dynamic(
                0,
                &light_buffer,
                size_of::<PointLightShadowUniform>() as u64,
            ),
        );

        // Get mesh descriptor layout
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

        let shadow_alpha_layout = ShadowResource::shadow_layout(rcx);

        // Create pipeline
        let pipeline_layout = rcx.device().create_pipeline_layout(&[
            light_layout.clone(),
            mesh_layout.clone(),
            shadow_alpha_layout,
        ]);

        let depth_mode = DepthMode::Texture(DepthStencilOptions {
            format: TextureFormat::Depth32,
            compare: DepthCompare::Less,
            write_enabled: true,
            depth_bias: Some(DepthBias {
                constant: 2,
                slope_scale: 4.0,
            }),
        });

        let mut pipeline: HashMap<CullMode, RenderPipeline> = HashMap::default();

        for cull_mode in [CullMode::None, CullMode::Back, CullMode::Front] {
            pipeline.insert(
                cull_mode,
                rcx.device().create_pipeline(PipelineCreateInfo {
                    label: Some("DirectionalShadowPass"),
                    layout: pipeline_layout.clone(),
                    shader: shader.clone(),
                    color_formats: &[],
                    depth: depth_mode.clone(),
                    cull_mode: cull_mode,
                    alpha_mode: AlphaMode::Opaque,
                    sample_count: 1,
                    vertex_buffer_layout: Some(Vertex::buffer_layout()),
                }),
            );
        }

        Self {
            light_buffer,
            light_descriptor,
            pipeline,
            mesh_descriptors: HashMap::default(),
            mesh_layout,
            mesh_buffers: HashMap::default(),
        }
    }
    fn draw(
        &mut self,
        rcx: &RenderContext,
        frame: &mut Frame,
        graph_ctx: &mut RenderGraphContext,
        game_ctx: &GameContext,
    ) {
        // Get shared resources (shadow resources arent created in this node)
        let cube_array = match graph_ctx.get_shared_resource::<TextureCubeArray>("point_shadows") {
            Some(array) => array,
            None => {
                log::error!("PointShadowPass: No point_shadows cube array found");
                return;
            }
        };

        let scene = &game_ctx.scene;

        // Get scene data
        let point_lights = scene.collect::<PointLight>();
        let mesh_instances = scene.collect::<MeshInstance3D>();

        if point_lights.is_empty() || mesh_instances.is_empty() {
            return;
        }

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

        let point_light_data = PointLightBuffer::from_lights(
            &point_lights
                .iter()
                .enumerate()
                .map(|(i, light)| light.read().get_buffered_data(i))
                .collect::<Vec<_>>(),
        );

        rcx.queue()
            .write_buffer(point_light_buffer, &point_light_data);

        // References to self fields
        let light_buffer = &self.light_buffer;
        let light_descriptor = &self.light_descriptor;

        let light_data: Vec<PointLightShadowUniform> = point_lights
            .iter()
            .map(|light| {
                light
                    .read()
                    .get_shadow_transformations()
                    .iter()
                    .map(|vp| {
                        let light_pos = light.read().transform.world_space().position;
                        PointLightShadowUniform {
                            view_projection: vp.to_cols_array_2d(),
                            light_pos: [light_pos.x, light_pos.y, light_pos.z, 0.0],
                            far_plane: PointLight::calculate_far_plane(
                                light.read().get_intensity(),
                                0.01,
                            ),
                            _padding: Zeroable::zeroed(),
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect();

        rcx.queue().write_buffer_slice(light_buffer, &light_data);

        let bundles = graph_ctx
            .get_shared_resource::<collect_mesh::BundledMeshes>("mesh_bundles")
            .unwrap();

        // Render each point light's cube map
        for (light_idx, light) in point_lights.iter().enumerate() {
            // Skip if light index exceeds array size
            if (light_idx as u32) >= cube_array.array_layers() {
                break;
            }

            // Get view-projection matrices for all 6 cube faces
            let shadow_transforms = light.read().get_shadow_transformations();

            // Render each cube face
            for (face_idx, vp_matrix) in CubeFace::iter().zip(shadow_transforms.iter()) {
                let layer = light_idx as u32 * 6 + face_idx as u32;
                let face_frustum = Frustum::from_view_proj(vp_matrix);

                let (batches, data) =
                    ShadowResource::cull_and_batch_meshes(&bundles.meshes, face_frustum);

                let buffer = self.mesh_buffers.entry(face_idx as u32).or_insert_with(|| {
                    rcx.device().create_sized_storage_buffer(
                        size_of::<Mesh3DUniformBufferData>() * MAX_MESH,
                    )
                });

                rcx.queue().write_buffer_slice(buffer, &data);

                let descriptor =
                    self.mesh_descriptors
                        .entry(face_idx as u32)
                        .or_insert_with(|| {
                            rcx.device().build_descriptor_set(
                                DescriptorSet::builder(&self.mesh_layout).storage(0, buffer),
                            )
                        });

                // Get depth texture for this cube face
                let face_view = cube_array.create_face_view(light_idx as u32, face_idx);

                // Render meshes to this cube face
                frame
                    .render(
                        RenderOptions {
                            label: Some("Point Shadow Pass"),
                            color_targets: &[],
                            depth_target: Some(&face_view),
                            clear_color: None,
                            clear_depth: Some(1.0),
                        },
                        |mut fb| {
                            fb.bind_descriptor_set_with_offset(
                                0,
                                light_descriptor,
                                &[size_of::<PointLightShadowUniform>() as u32 * layer],
                            )
                            .bind_descriptor_set(1, &descriptor);

                            for material_batch in batches {
                                // fb.bind_descriptor_set(3, &material_batch.descriptor);
                                fb.use_pipeline(
                                    self.pipeline.get(&material_batch.cull_mode).unwrap(),
                                );

                                fb.bind_descriptor_set(2, &material_batch.shadow_descriptor);

                                for mesh_batch in material_batch.meshes {
                                    fb.bind_vertex_buffer(&mesh_batch.mesh.get_vertex_buffer())
                                        .bind_index_buffer(&mesh_batch.mesh.get_index_buffer())
                                        .draw_indexed(mesh_batch.start..mesh_batch.end);
                                }
                            }
                        },
                    )
                    .expect("failed to render point shadow cube face");
            }
        }
    }
}
