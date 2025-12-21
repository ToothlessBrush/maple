use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use maple_engine::{Scene, utils::Debug};
use maple_renderer::{
    core::{
        Buffer, DepthCompare, RenderContext, StageFlags,
        descriptor_set::{
            DescriptorBindingType, DescriptorSet, DescriptorSetLayout,
            DescriptorSetLayoutDescriptor,
        },
        texture::{TextureArray, TextureCreateInfo, TextureFormat, TextureUsage},
    },
    render_graph::{
        graph::RenderGraphContext,
        node::{DepthTarget, RenderNode, RenderNodeContext, RenderNodeDescriptor},
    },
};

use crate::nodes::{camera::Camera3D, directional_light::DirectionalLight, mesh::Mesh3D};

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
    // Descriptor layout for light VP matrix
    light_vp_layout: Option<DescriptorSetLayout>,

    // Buffer for light view-projection matrix
    light_vp_buffer: Option<Buffer<LightVPUniform>>,

    // Descriptor set for light VP
    light_vp_descriptor: Option<DescriptorSet>,
}

impl Default for DirectionalShadowPass {
    fn default() -> Self {
        Self {
            light_vp_layout: None,
            light_vp_buffer: None,
            light_vp_descriptor: None,
        }
    }
}

impl RenderNode for DirectionalShadowPass {
    fn setup(
        &mut self,
        render_ctx: &RenderContext,
        _graph_ctx: &mut RenderGraphContext,
    ) -> RenderNodeDescriptor {
        // Create depth-only shader (no fragment output, just depth write)
        let shader = render_ctx.create_shader_pair(maple_renderer::core::ShaderPair::Wgsl {
            vert: include_str!("../../res/shaders/directional_shadow/directional_shadow.vert.wgsl"),
            frag: include_str!("../../res/shaders/directional_shadow/directional_shadow.frag.wgsl"),
        });

        // Create descriptor set layout for light VP matrix
        let light_vp_layout =
            render_ctx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
                label: Some("DirectionalShadow_LightVP"),
                visibility: StageFlags::VERTEX,
                layout: &[DescriptorBindingType::UniformBuffer], // Binding 0: light VP
            });

        // Create buffer for light VP matrix
        let light_vp_buffer = render_ctx.create_uniform_buffer(&LightVPUniform {
            view_projection: Mat4::IDENTITY.to_cols_array_2d(),
        });

        // Build descriptor set
        let light_vp_descriptor = render_ctx.build_descriptor_set(
            DescriptorSet::builder(&light_vp_layout).uniform(0, &light_vp_buffer),
        );

        self.light_vp_layout = Some(light_vp_layout.clone());
        self.light_vp_buffer = Some(light_vp_buffer);
        self.light_vp_descriptor = Some(light_vp_descriptor);

        // Get mesh descriptor layout
        let mesh_layout = Mesh3D::layout(render_ctx).clone();

        // Create a placeholder depth texture (will be updated in draw())
        let placeholder_depth = render_ctx.create_texture(TextureCreateInfo {
            label: Some("directional_shadow_placeholder_depth"),
            width: 1,
            height: 1,
            format: TextureFormat::Depth32,
            usage: TextureUsage::RENDER_ATTACHMENT,
        });

        RenderNodeDescriptor {
            shader,
            descriptor_set_layouts: vec![light_vp_layout, mesh_layout],
            target: vec![], // No color target, depth only
            depth: DepthTarget::Texture {
                depth_texture: placeholder_depth,
                compare_function: DepthCompare::Less,
                depth_bias: None, // (constant, slope) - helps prevent shadow acne
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
        let shadow_array =
            match graph_ctx.get_shared_resource::<TextureArray>("directional_shadows") {
                Some(array) => array,
                None => return, // No shadows to render
            };

        // Get scene data
        let directional_lights = scene.collect_items::<DirectionalLight>();
        let meshes = scene.collect_items::<Mesh3D>();
        let cameras = scene.collect_items::<Camera3D>();

        if directional_lights.is_empty() || meshes.is_empty() || cameras.is_empty() {
            return;
        }

        // Get active camera for light view centering
        let Some(camera) = cameras
            .iter()
            .filter(|c| c.is_active)
            .max_by_key(|c| c.priority)
        else {
            Debug::print_once("no active camera in scene");
            return;
        };

        // References to self fields
        let light_vp_buffer = self.light_vp_buffer.as_ref().unwrap();
        let light_vp_descriptor = self.light_vp_descriptor.as_ref().unwrap();

        // Render each directional light's cascades
        for (light_idx, light) in directional_lights.iter().enumerate() {
            // Get view-projection matrices for all cascades
            let vp_matrices = light.view_projection(camera, render_ctx.aspect_ratio());

            // Render each cascade
            for (cascade_idx, vp_matrix) in vp_matrices.iter().enumerate() {
                // Calculate layer index: light_idx * 4 + cascade_idx
                let layer = (light_idx * 4 + cascade_idx) as u32;

                // Skip if layer exceeds array size
                if layer >= shadow_array.array_layers() {
                    break;
                }

                // Update light VP buffer
                let light_vp_uniform = LightVPUniform {
                    view_projection: vp_matrix.to_cols_array_2d(),
                };
                render_ctx.write_buffer(light_vp_buffer, &light_vp_uniform);

                // Update depth texture to this cascade layer
                let layer_texture = shadow_array.create_layer_texture(layer);
                node_ctx.update_depth_texture(layer_texture);

                // Render meshes to this cascade
                render_ctx
                    .render(node_ctx, |mut fb| {
                        fb.bind_descriptor_set(0, light_vp_descriptor);

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
                    .expect("failed to render directional shadow cascade");
            }
        }
    }
}
