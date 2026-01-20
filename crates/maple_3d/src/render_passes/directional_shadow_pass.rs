use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use maple_engine::{GameContext, Scene};
use maple_renderer::{
    core::{
        Buffer, CullMode, DepthCompare, DepthStencilOptions, RenderContext, StageFlags,
        context::RenderOptions,
        descriptor_set::{DescriptorBindingType, DescriptorSet, DescriptorSetLayoutDescriptor},
        pipeline::{AlphaMode, PipelineCreateInfo, RenderPipeline},
        texture::{TextureArray, TextureFormat},
    },
    render_graph::{
        graph::RenderGraphContext,
        node::{DepthMode, RenderNode},
    },
};

use crate::{
    components::material::MaterialProperties,
    math::Frustum,
    nodes::{camera::Camera3D, directional_light::DirectionalLight, mesh::Mesh3D},
};

/// Uniform buffer for light view-projection matrix
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct LightVPUniform {
    view_projection: [[f32; 4]; 4],
}

/// Directional shadow pass renders depth from directional light perspectives
///
/// This pass renders each directional light's cascaded shadow maps by:
/// 1. Getting the light's view-projection matrices (up to 4 cascades)
/// 2. Rendering all meshes from the light's perspective to depth layers
/// 3. Storing depth values for shadow sampling in the main pass
pub struct DirectionalShadowPass {
    // Buffer for light view-projection matrix
    light_vp_buffer: Buffer<LightVPUniform>,

    // Descriptor set for light VP
    light_vp_descriptor: DescriptorSet,

    // Render pipeline
    pipeline: RenderPipeline,
}

impl DirectionalShadowPass {
    pub fn setup(rcx: &RenderContext, _: &mut RenderGraphContext) -> Self {
        // Create depth-only shader (no fragment output, just depth write)
        let shader = rcx.create_shader_pair(maple_renderer::core::ShaderPair::Wgsl {
            vert: include_str!("../../res/shaders/directional_shadow/directional_shadow.vert.wgsl"),
            frag: include_str!("../../res/shaders/directional_shadow/directional_shadow.frag.wgsl"),
        });

        // Create descriptor set layout for light VP matrix
        let light_vp_layout = rcx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
            label: Some("DirectionalShadow_LightVP"),
            visibility: StageFlags::VERTEX,
            layout: &[DescriptorBindingType::UniformBuffer], // Binding 0: light VP
        });

        // Create buffer for light VP matrix
        let light_vp_buffer = rcx.create_uniform_buffer(&LightVPUniform {
            view_projection: Mat4::IDENTITY.to_cols_array_2d(),
        });

        // Build descriptor set
        let light_vp_descriptor = rcx.build_descriptor_set(
            DescriptorSet::builder(&light_vp_layout).uniform(0, &light_vp_buffer),
        );

        // Get mesh descriptor layout
        let mesh_layout = Mesh3D::layout(rcx).clone();

        // Get material descriptor layout
        let material_layout = MaterialProperties::layout(rcx).clone();

        // Create pipeline
        let pipeline_layout = rcx.create_pipeline_layout(&[
            light_vp_layout.clone(),
            mesh_layout.clone(),
            material_layout.clone(),
        ]);

        let depth_mode = DepthMode::Texture(DepthStencilOptions {
            format: TextureFormat::Depth32,
            compare: DepthCompare::Less,
            write_enabled: true,
            depth_bias: Some((2.0, 2.5)),
        });

        let pipeline = rcx.create_pipeline(PipelineCreateInfo {
            label: Some("DirectionalShadowPass"),
            layout: pipeline_layout,
            shader: shader.clone(),
            color_formats: &[],
            depth: &depth_mode,
            cull_mode: CullMode::Back,
            alpha_mode: AlphaMode::Opaque,
            sample_count: 1,
            use_vertex_buffer: true,
        });

        Self {
            light_vp_buffer,
            light_vp_descriptor,
            pipeline,
        }
    }
}

impl RenderNode for DirectionalShadowPass {
    fn draw(
        &mut self,
        render_ctx: &RenderContext,
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
        let meshes = scene.collect::<Mesh3D>();
        let cameras = scene.collect::<Camera3D>();

        if directional_lights.is_empty() || meshes.is_empty() || cameras.is_empty() {
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

        // References to self fields
        let light_vp_buffer = &self.light_vp_buffer;
        let light_vp_descriptor = &self.light_vp_descriptor;
        let pipeline = &self.pipeline;

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

                // Update light VP buffer
                let light_vp_uniform = LightVPUniform {
                    view_projection: vp_matrix.to_cols_array_2d(),
                };
                render_ctx.write_buffer(light_vp_buffer, &light_vp_uniform);

                // Get depth texture for this cascade layer
                let layer_view = shadow_array.create_layer_view(layer);

                // Render meshes to this cascade
                render_ctx
                    .render(
                        RenderOptions {
                            label: Some(&format!("Cascade: {} Pass", cascade_idx)),
                            color_targets: &[],
                            depth_target: Some(&layer_view),
                            clear_color: None,
                            clear_depth: Some(1.0),
                        },
                        |mut fb| {
                            fb.use_pipeline(pipeline)
                                .bind_descriptor_set(0, light_vp_descriptor);

                            for mesh in &meshes {
                                let mesh = mesh.read();
                                let Some(material) = mesh
                                    .get_material()
                                    .get_descriptor(render_ctx, &game_ctx.assets)
                                else {
                                    continue;
                                };
                                if !cascade_fustum.intersects_aabb(&mesh.world_aabb()) {
                                    continue;
                                }
                                let mesh_descriptor = mesh.get_descriptor(render_ctx);
                                let vertex_buffer = mesh.get_vertex_buffer(render_ctx);
                                let index_buffer = mesh.get_index_buffer(render_ctx);

                                fb.bind_descriptor_set(1, &mesh_descriptor)
                                    .bind_descriptor_set(2, &material)
                                    .bind_vertex_buffer(&vertex_buffer)
                                    .bind_index_buffer(&index_buffer)
                                    .draw_indexed();
                            }
                        },
                    )
                    .expect("failed to render directional shadow cascade");
            }
        }
    }
}
