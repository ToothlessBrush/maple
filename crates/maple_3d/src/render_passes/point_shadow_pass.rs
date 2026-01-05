use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use maple_engine::Scene;
use maple_renderer::{
    core::{
        Buffer, CullMode, DepthCompare, DepthStencilOptions, RenderContext, StageFlags,
        context::RenderOptions,
        descriptor_set::{
            DescriptorBindingType, DescriptorSet, DescriptorSetLayout,
            DescriptorSetLayoutDescriptor,
        },
        pipeline::{AlphaMode, PipelineCreateInfo, RenderPipeline},
        texture::{CubeFace, TextureCubeArray, TextureFormat},
    },
    render_graph::{
        graph::RenderGraphContext,
        node::{DepthMode, RenderNode},
    },
};

use crate::{
    components::material::MaterialProperties,
    nodes::{mesh::Mesh3D, point_light::PointLight},
};

/// Uniform buffer for point light shadow data
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct PointLightShadowUniform {
    view_projection: [[f32; 4]; 4], // 64 bytes
    light_pos: [f32; 4],            // 16 bytes
    far_plane: f32,                 // 4 bytes
    _padding: [f32; 7],             // 28 bytes (total: 112 bytes to match WGSL alignment)
}

/// Point shadow pass renders depth from point light perspectives to cube maps
///
/// This pass renders each point light's shadow cube map by:
/// 1. Getting the light's 6 view-projection matrices (one per cube face)
/// 2. Rendering all meshes from each face's perspective
/// 3. Storing depth values for shadow sampling in the main pass
#[derive(Default)]
pub struct PointShadowPass {
    // Descriptor layout for light data
    light_layout: Option<DescriptorSetLayout>,

    // Buffer for light shadow data
    light_buffer: Option<Buffer<PointLightShadowUniform>>,

    // Descriptor set for light data
    light_descriptor: Option<DescriptorSet>,

    // Render pipeline
    pipeline: Option<RenderPipeline>,
}

impl RenderNode for PointShadowPass {
    fn setup(&mut self, render_ctx: &RenderContext, _graph_ctx: &mut RenderGraphContext) {
        // Create depth-only shader
        let shader = render_ctx.create_shader_pair(maple_renderer::core::ShaderPair::Wgsl {
            vert: include_str!("../../res/shaders/point_shadow/point_shadow.vert.wgsl"),
            frag: include_str!("../../res/shaders/point_shadow/point_shadow.frag.wgsl"),
        });

        // Create descriptor set layout for light data
        let light_layout = render_ctx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
            label: Some("PointShadow_Light"),
            visibility: StageFlags::VERTEX | StageFlags::FRAGMENT,
            layout: &[DescriptorBindingType::UniformBuffer], // Binding 0: light data
        });

        // Create buffer for light data
        let light_buffer = render_ctx.create_uniform_buffer(&PointLightShadowUniform {
            view_projection: Mat4::IDENTITY.to_cols_array_2d(),
            light_pos: [0.0; 4],
            far_plane: 10.0,
            _padding: [0.0; 7],
        });

        // Build descriptor set
        let light_descriptor = render_ctx
            .build_descriptor_set(DescriptorSet::builder(&light_layout).uniform(0, &light_buffer));

        self.light_layout = Some(light_layout.clone());
        self.light_buffer = Some(light_buffer);
        self.light_descriptor = Some(light_descriptor);

        // Get mesh descriptor layout
        let mesh_layout = Mesh3D::layout(render_ctx).clone();

        // Get material descriptor layout
        let material_layout = MaterialProperties::layout(render_ctx).clone();

        // Create pipeline
        let pipeline_layout = render_ctx.create_pipeline_layout(&[
            light_layout.clone(),
            mesh_layout.clone(),
            material_layout.clone(),
        ]);

        let depth_mode = DepthMode::Texture(DepthStencilOptions {
            format: TextureFormat::Depth32,
            compare: DepthCompare::Less,
            write_enabled: true,
            depth_bias: Some((2.0, 4.0)),
        });

        let pipeline = render_ctx.create_pipeline(PipelineCreateInfo {
            label: Some("PointShadowPass"),
            layout: pipeline_layout,
            shader: shader.clone(),
            color_formats: &[],
            depth: &depth_mode,
            cull_mode: CullMode::Front,
            alpha_mode: AlphaMode::Opaque,
            sample_count: 1,
            use_vertex_buffer: true,
        });

        self.pipeline = Some(pipeline);
    }

    fn draw(
        &mut self,
        render_ctx: &RenderContext,
        graph_ctx: &mut RenderGraphContext,
        scene: &Scene,
    ) {
        // Get shared resources
        let cube_array = match graph_ctx.get_shared_resource::<TextureCubeArray>("point_shadows") {
            Some(array) => array,
            None => {
                log::error!("PointShadowPass: No point_shadows cube array found");
                return;
            }
        };

        // Get scene data
        let point_lights = scene.collect::<PointLight>();
        let meshes = scene.collect::<Mesh3D>();

        if point_lights.is_empty() || meshes.is_empty() {
            return;
        }

        // References to self fields
        let light_buffer = self.light_buffer.as_ref().unwrap();
        let light_descriptor = self.light_descriptor.as_ref().unwrap();
        let Some(pipeline) = &self.pipeline else {
            return;
        };

        // Render each point light's cube map
        for (light_idx, light) in point_lights.iter().enumerate() {
            // Skip if light index exceeds array size
            if (light_idx as u32) >= cube_array.array_layers() {
                break;
            }

            // Get the light's position
            let light_pos = light.read().transform.world_space().position;

            // Get view-projection matrices for all 6 cube faces
            let shadow_transforms = light.read().get_shadow_transformations();
            let far_plane = PointLight::calculate_far_plane(light.read().get_intensity(), 0.01);

            // Render each cube face
            for (face_idx, vp_matrix) in CubeFace::iter().zip(shadow_transforms.iter()) {
                // Update light buffer
                let light_uniform = PointLightShadowUniform {
                    view_projection: vp_matrix.to_cols_array_2d(),
                    light_pos: [light_pos.x, light_pos.y, light_pos.z, 0.0],
                    far_plane,
                    _padding: [0.0; 7],
                };
                render_ctx.write_buffer(light_buffer, &light_uniform);

                // Get depth texture for this cube face
                let face_view = cube_array.create_face_view(light_idx as u32, face_idx);

                // Render meshes to this cube face
                render_ctx
                    .render(
                        RenderOptions {
                            label: Some("Point Shadow Pass"),
                            color_targets: &[],
                            depth_target: Some(&face_view),
                            clear_color: None,
                        },
                        |mut fb| {
                            fb.use_pipeline(pipeline)
                                .bind_descriptor_set(0, light_descriptor);

                            for mesh in &meshes {
                                let mesh = mesh.read();
                                let mesh_descriptor = mesh.get_descriptor(render_ctx);
                                let material_descriptor =
                                    mesh.get_material().get_descriptor(render_ctx);
                                let vertex_buffer = mesh.get_vertex_buffer(render_ctx);
                                let index_buffer = mesh.get_index_buffer(render_ctx);

                                fb.bind_descriptor_set(1, &mesh_descriptor)
                                    .bind_descriptor_set(2, &material_descriptor)
                                    .bind_vertex_buffer(&vertex_buffer)
                                    .bind_index_buffer(&index_buffer)
                                    .draw_indexed();
                            }
                        },
                    )
                    .expect("failed to render point shadow cube face");
            }
        }
    }
}
