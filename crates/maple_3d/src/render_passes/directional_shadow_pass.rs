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
        texture::{TextureArray, TextureFormat},
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
        camera::Camera3D,
        directional_light::{DirectionalLight, DirectionalLightBuffer},
        mesh_instance::{Mesh3DUniformBufferData, MeshInstance3D},
    },
    render_passes::{
        collect_mesh::BundledMeshes,
        main_pass::MAX_MESH,
        shadow_resource::{self, ShadowResource},
    },
};

/// Uniform buffer for light view-projection matrix
///
/// the standard alignment is 256 bytes for offset
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct LightVPUniform {
    view_projection: [[f32; 4]; 4],
    _padding: [u8; 192],
}

/// Directional shadow pass renders depth from directional light perspectives
///
/// This pass renders each directional light's cascaded shadow maps by:
/// 1. Getting the light's view-projection matrices (up to 4 cascades)
/// 2. Rendering all meshes from the light's perspective to depth layers
/// 3. Storing depth values for shadow sampling in the main pass
pub struct DirectionalShadowPass {
    // Buffer for light view-projection matrix
    light_vp_buffer: Buffer<[LightVPUniform]>,

    // Descriptor set for light VP
    light_vp_descriptor: DescriptorSet,

    // Render pipeline
    pipeline: HashMap<CullMode, RenderPipeline>,

    mesh_buffers: HashMap<u32, Buffer<[Mesh3DUniformBufferData]>>,
    mesh_layout: DescriptorSetLayout,
    mesh_descriptors: HashMap<u32, DescriptorSet>,
}

impl DirectionalShadowPass {}

impl RenderNode for DirectionalShadowPass {
    fn stage(&self) -> Stage {
        Stage::Shadow
    }

    fn setup(rcx: &RenderContext, _: &mut RenderGraphContext) -> Self {
        let shader = GraphicsShader {
            vertex: rcx
                .device()
                .compile_shader(
                    include_str!(
                        "../../res/shaders/directional_shadow/directional_shadow.vert.wgsl"
                    )
                    .into(),
                )
                .expect("directional shadow vert shader to compile"),
            fragment: rcx
                .device()
                .compile_shader(
                    include_str!(
                        "../../res/shaders/directional_shadow/directional_shadow.frag.wgsl"
                    )
                    .into(),
                )
                .expect("directional frag shader to compile"),
        };

        // Create descriptor set layout for light VP matrix
        let light_vp_layout =
            rcx.device()
                .create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
                    label: Some("DirectionalShadow_LightVP"),
                    visibility: StageFlags::VERTEX,
                    layout: &[DescriptorBindingType::Storage {
                        read_only: true,
                        has_dynamic_offset: true,
                        min_size: Some(size_of::<LightVPUniform>()),
                    }], // Binding 0: light VP
                });

        // Create buffer for light VP matrix
        let light_vp_buffer = rcx.device().create_sized_storage_buffer(
            size_of::<LightVPUniform>() * shadow_resource::DIRECTIONAL_SHADOW_SIZE as usize,
        );

        // Build descriptor set
        let light_vp_descriptor = rcx.device().build_descriptor_set(
            DescriptorSet::builder(&light_vp_layout).storage_dynamic(
                0,
                &light_vp_buffer,
                size_of::<LightVPUniform>() as u64,
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

        // Get material descriptor layout
        // let material_layout = MaterialProperties::layout(rcx).clone();

        // Create pipeline
        let pipeline_layout = rcx.device().create_pipeline_layout(&[
            light_vp_layout.clone(),
            mesh_layout.clone(),
            // material_layout.clone(),
        ]);

        let depth_mode = DepthMode::Texture(DepthStencilOptions {
            format: TextureFormat::Depth32,
            compare: DepthCompare::Less,
            write_enabled: true,
            depth_bias: Some(DepthBias {
                constant: 2,
                slope_scale: 2.5,
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
            light_vp_buffer,
            light_vp_descriptor,
            pipeline,
            mesh_buffers: HashMap::new(),
            mesh_layout,
            mesh_descriptors: HashMap::new(),
        }
    }

    fn draw(
        &mut self,
        render_ctx: &RenderContext,
        frame: &mut Frame,
        graph_ctx: &mut RenderGraphContext,
        game_ctx: &GameContext,
    ) {
        // Get shared resources
        let shadow_array =
            match graph_ctx.get_shared_resource::<TextureArray>("directional_shadows") {
                Some(array) => array,
                None => return, // No shadows to render
            };

        let scene = &game_ctx.scene;

        // Get scene data
        let directional_lights = scene.collect::<DirectionalLight>();
        let mesh_instance = scene.collect::<MeshInstance3D>();
        let cameras = scene.collect::<Camera3D>();

        if directional_lights.is_empty() || mesh_instance.is_empty() || cameras.is_empty() {
            return;
        }

        // Get active camera for light view centering
        let Some(camera) = cameras
            .iter()
            .filter(|c| c.read().is_active)
            .max_by_key(|c| c.read().priority)
        else {
            return;
        };

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

        // Update light buffers with current scene data
        let direct_light_data = DirectionalLightBuffer::from_lights(
            &directional_lights
                .iter()
                .enumerate()
                .map(|(i, light)| {
                    light
                        .read()
                        .to_buffer_data(&camera.read(), render_ctx.aspect_ratio(), i)
                })
                .collect::<Vec<_>>(),
        );

        render_ctx
            .queue()
            .write_buffer(direct_light_buffer, &direct_light_data);

        // References to self fields
        let light_vp_buffer = &self.light_vp_buffer;
        let light_vp_descriptor = &self.light_vp_descriptor;

        let light_data: Vec<LightVPUniform> = directional_lights
            .iter()
            .map(|light| {
                let vp = light
                    .read()
                    .view_projection(&camera.read(), render_ctx.aspect_ratio());
                vp.iter()
                    .map(|mat| LightVPUniform {
                        view_projection: mat.to_cols_array_2d(),
                        _padding: Zeroable::zeroed(),
                    })
                    .collect::<Vec<_>>()
            })
            .flatten()
            .collect();

        render_ctx
            .queue()
            .write_buffer_slice(light_vp_buffer, &light_data);

        let bundles = graph_ctx
            .get_shared_resource::<BundledMeshes>("mesh_bundles")
            .unwrap();

        // Render each directional light's cascades
        for (light_idx, light) in directional_lights.iter().enumerate() {
            // Get view-projection matrices for all cascades
            let vp_matrices = light
                .read()
                .view_projection(&camera.read(), render_ctx.aspect_ratio());

            // Render each cascade
            for (cascade_idx, vp_matrix) in vp_matrices.iter().enumerate() {
                // Calculate layer index: light_idx * 4 + cascade_idx
                let layer = (light_idx * 4 + cascade_idx) as u32;

                let cascade_fustum = Frustum::from_view_proj(vp_matrix);

                // Skip if layer exceeds array size
                if layer >= shadow_array.array_layers() {
                    break;
                }

                let (batches, data) =
                    ShadowResource::cull_and_batch_meshes(&bundles.meshes, cascade_fustum);

                let buffer = self.mesh_buffers.entry(cascade_idx as u32).or_insert(
                    render_ctx.device().create_sized_storage_buffer(
                        size_of::<Mesh3DUniformBufferData>() * MAX_MESH,
                    ),
                );

                render_ctx.queue().write_buffer_slice(buffer, &data);

                let descriptor = self.mesh_descriptors.entry(cascade_idx as u32).or_insert(
                    render_ctx.device().build_descriptor_set(
                        DescriptorSet::builder(&self.mesh_layout).storage(0, buffer),
                    ),
                );

                // Get depth texture for this cascade layer
                let layer_view = shadow_array.create_layer_view(layer);

                // Render meshes to this cascade
                frame
                    .render(
                        RenderOptions {
                            label: Some(&format!("Cascade: {} Pass", cascade_idx)),
                            color_targets: &[],
                            depth_target: Some(&layer_view),
                            clear_color: None,
                            clear_depth: Some(1.0),
                        },
                        |mut fb| {
                            fb.bind_descriptor_set_with_offset(
                                0,
                                light_vp_descriptor,
                                &[size_of::<LightVPUniform>() as u32 * layer],
                            )
                            .bind_descriptor_set(1, &descriptor);

                            for material_batch in batches {
                                // fb.bind_descriptor_set(3, &material_batch.descriptor);
                                fb.use_pipeline(
                                    self.pipeline.get(&material_batch.cull_mode).unwrap(),
                                );

                                for mesh_batch in material_batch.meshes {
                                    fb.bind_vertex_buffer(&mesh_batch.mesh.get_vertex_buffer())
                                        .bind_index_buffer(&mesh_batch.mesh.get_index_buffer())
                                        .draw_indexed(mesh_batch.start..mesh_batch.end);
                                }
                            }
                        },
                    )
                    .expect("failed to render directional shadow cascade");
            }
        }
    }
}
