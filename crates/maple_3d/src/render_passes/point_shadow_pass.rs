use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use maple_engine::Scene;
use maple_renderer::{
    core::{
        Buffer, DepthCompare, RenderContext, StageFlags,
        descriptor_set::{
            DescriptorBindingType, DescriptorSet, DescriptorSetLayout,
            DescriptorSetLayoutDescriptor,
        },
        texture::{TextureCreateInfo, TextureCubeArray, TextureFormat, TextureUsage},
    },
    render_graph::{
        graph::RenderGraphContext,
        node::{DepthTarget, RenderNode, RenderNodeContext, RenderNodeDescriptor},
    },
};

use crate::nodes::{mesh::Mesh3D, point_light::PointLight};

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
}

impl RenderNode for PointShadowPass {
    fn setup(
        &mut self,
        render_ctx: &RenderContext,
        _graph_ctx: &mut RenderGraphContext,
    ) -> RenderNodeDescriptor {
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

        // Create a placeholder depth texture (will be updated in draw())
        let placeholder_depth = render_ctx.create_texture(TextureCreateInfo {
            label: Some("point_shadow_placeholder_depth"),
            width: 1,
            height: 1,
            format: TextureFormat::Depth32,
            usage: TextureUsage::RENDER_ATTACHMENT,
        });

        RenderNodeDescriptor {
            shader,
            descriptor_set_layouts: vec![light_layout, mesh_layout],
            target: vec![], // No color target, depth only
            depth: DepthTarget::Texture {
                depth_texture: placeholder_depth,
                compare_function: DepthCompare::Less,
                depth_bias: Some((2.0, 4.0)), // Depth bias for point light shadows
            },
        }
    }

    fn draw(
        &mut self,
        render_ctx: &RenderContext,
        node_ctx: &mut RenderNodeContext,
        graph_ctx: &mut RenderGraphContext,
        scene: &Scene,
    ) {
        // Get shared resources
        let cube_array = match graph_ctx.get_shared_resource::<TextureCubeArray>("point_shadows") {
            Some(array) => array,
            None => {
                println!("PointShadowPass: No point_shadows cube array found");
                return;
            }
        };

        // Get scene data
        let point_lights = scene.collect_items::<PointLight>();
        let meshes = scene.collect_items::<Mesh3D>();

        if point_lights.is_empty() || meshes.is_empty() {
            return;
        }

        // References to self fields
        let light_buffer = self.light_buffer.as_ref().unwrap();
        let light_descriptor = self.light_descriptor.as_ref().unwrap();

        // Render each point light's cube map
        for (light_idx, light) in point_lights.iter().enumerate() {
            // Skip if light index exceeds array size
            if (light_idx as u32) >= cube_array.array_layers() {
                break;
            }

            // Get the light's position
            let light_pos = light.transform.world_space().position;

            // Get view-projection matrices for all 6 cube faces
            let shadow_transforms = light.get_shadow_transformations();
            let far_plane = PointLight::calculate_far_plane(light.get_intensity(), 0.01);

            // Render each cube face
            for (face_idx, vp_matrix) in shadow_transforms.iter().enumerate() {
                // Update light buffer
                let light_uniform = PointLightShadowUniform {
                    view_projection: vp_matrix.to_cols_array_2d(),
                    light_pos: [light_pos.x, light_pos.y, light_pos.z, 0.0],
                    far_plane,
                    _padding: [0.0; 7],
                };
                render_ctx.write_buffer(light_buffer, &light_uniform);

                // Update depth texture to this cube face
                let face_texture =
                    cube_array.create_face_texture(light_idx as u32, face_idx as u32);
                node_ctx.update_depth_texture(face_texture);

                // Render meshes to this cube face
                render_ctx
                    .render(node_ctx, |mut fb| {
                        fb.bind_descriptor_set(0, light_descriptor);

                        for mesh in &meshes {
                            let mesh_descriptor = mesh.get_descriptor(render_ctx);
                            let vertex_buffer = mesh.get_vertex_buffer(render_ctx);
                            let index_buffer = mesh.get_index_buffer(render_ctx);

                            fb.bind_descriptor_set(1, &mesh_descriptor)
                                .bind_vertex_buffer(&vertex_buffer)
                                .bind_index_buffer(&index_buffer)
                                .draw_indexed();
                        }
                    })
                    .expect("failed to render point shadow cube face");
            }
        }
    }
}
