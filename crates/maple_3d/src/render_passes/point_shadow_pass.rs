use std::mem::zeroed;

use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use maple_engine::{GameContext, asset::AssetState};
use maple_renderer::{
    core::{
        Buffer, CullMode, DepthBias, DepthCompare, DepthStencilOptions, Frame, GraphicsShader,
        RenderContext, StageFlags,
        context::RenderOptions,
        descriptor_set::{DescriptorBindingType, DescriptorSet, DescriptorSetLayoutDescriptor},
        pipeline::{AlphaMode, PipelineCreateInfo, RenderPipeline},
        texture::{CubeFace, TextureCubeArray, TextureFormat},
    },
    render_graph::{
        graph::RenderGraphContext,
        node::{DepthMode, RenderNode},
    },
};

use crate::{
    assets::mesh::Mesh3D,
    math::Frustum,
    nodes::{mesh_instance::MeshInstance3D, point_light::PointLight},
    render_passes::shadow_resource,
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
    pipeline: RenderPipeline,
}

impl PointShadowPass {}

impl RenderNode for PointShadowPass {
    fn setup(rcx: &RenderContext, _gcx: &mut RenderGraphContext) -> Self {
        let shader = GraphicsShader {
            vertex: rcx
                .device()
                .compile_shader(
                    include_str!("../../res/shaders/point_shadow/point_shadow.vert.wgsl").into(),
                )
                .expect("compiled vertex shader"),
            fragment: rcx
                .device()
                .compile_shader(
                    include_str!("../../res/shaders/point_shadow/point_shadow.frag.wgsl").into(),
                )
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
        let mesh_layout = MeshInstance3D::layout(rcx).clone();

        // Get material descriptor layout
        // let material_layout = MaterialProperties::layout(rcx).clone();

        // Create pipeline
        let pipeline_layout = rcx.device().create_pipeline_layout(&[
            light_layout.clone(),
            mesh_layout.clone(),
            // material_layout.clone(),
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

        let pipeline = rcx.device().create_pipeline(PipelineCreateInfo {
            label: Some("PointShadowPass"),
            layout: pipeline_layout,
            shader: shader.clone(),
            color_formats: &[],
            depth: depth_mode,
            cull_mode: CullMode::Front,
            alpha_mode: AlphaMode::Opaque,
            sample_count: 1,
            use_vertex_buffer: true,
        });

        Self {
            light_buffer,
            light_descriptor,
            pipeline,
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
        let meshes = scene.collect::<MeshInstance3D>();

        if point_lights.is_empty() || meshes.is_empty() {
            return;
        }

        // References to self fields
        let light_buffer = &self.light_buffer;
        let light_descriptor = &self.light_descriptor;
        let pipeline = &self.pipeline;

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

        // Render each point light's cube map
        for (light_idx, light) in point_lights.iter().enumerate() {
            // Skip if light index exceeds array size
            if (light_idx as u32) >= cube_array.array_layers() {
                break;
            }

            // Get the light's position

            // Get view-projection matrices for all 6 cube faces
            let shadow_transforms = light.read().get_shadow_transformations();

            // Render each cube face
            for (face_idx, vp_matrix) in CubeFace::iter().zip(shadow_transforms.iter()) {
                // Update light buffer
                // let light_uniform = PointLightShadowUniform {
                //     view_projection: vp_matrix.to_cols_array_2d(),
                //     light_pos: [light_pos.x, light_pos.y, light_pos.z, 0.0],
                //     far_plane,
                //     _padding: [0.0; 7],
                // };
                // rcx.queue().write_buffer(light_buffer, &light_uniform);

                let layer = light_idx as u32 * 6 + face_idx as u32;
                let face_frustum = Frustum::from_view_proj(vp_matrix);

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
                            fb.use_pipeline(pipeline).bind_descriptor_set_with_offset(
                                0,
                                light_descriptor,
                                &[size_of::<PointLightShadowUniform>() as u32 * layer],
                            );

                            for mesh in &meshes {
                                let mesh_instance = mesh.read();

                                if !mesh_instance
                                    .material
                                    .as_ref()
                                    .and_then(|mat| match game_ctx.assets.get(mat) {
                                        AssetState::Loaded(inst) => Some(inst.casts_shadows()),
                                        _ => None,
                                    })
                                    .unwrap_or(false)
                                {
                                    continue;
                                }

                                let Some(mesh) = mesh_instance.mesh.clone() else {
                                    continue;
                                };
                                let AssetState::Loaded(mesh) = game_ctx.assets.get(&mesh) else {
                                    continue;
                                };

                                if !face_frustum.intersects_aabb(
                                    &mesh.world_aabb(mesh_instance.transform.world_space().clone()),
                                ) {
                                    continue;
                                }
                                let mesh_descriptor = mesh_instance.get_descriptor(rcx);
                                let vertex_buffer = mesh.get_vertex_buffer();
                                let index_buffer = mesh.get_index_buffer();

                                fb.bind_descriptor_set(1, &mesh_descriptor)
                                    // .bind_descriptor_set(2, &material)
                                    .bind_vertex_buffer(&vertex_buffer)
                                    .bind_index_buffer(&index_buffer)
                                    .draw_indexed(0);
                            }
                        },
                    )
                    .expect("failed to render point shadow cube face");
            }
        }
    }
}
