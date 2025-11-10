use std::sync::OnceLock;

use maple_engine::Scene;
use maple_renderer::{
    core::{
        Buffer, DescriptorBindingType, DescriptorSet, DescriptorSetLayout,
        DescriptorSetLayoutDescriptor, RenderContext, StageFlags,
        texture::{
            FilterMode, Sampler, SamplerOptions, TextureArray, TextureArrayCreateInfo,
            TextureCubeArray, TextureCubeArrayCreateInfo, TextureFormat, TextureMode, TextureUsage,
        },
    },
    render_graph::{
        graph::RenderGraphContext,
        node::{DepthTarget, RenderNode, RenderNodeContext, RenderNodeDescriptor},
    },
};

use crate::nodes::{
    directional_light::{DirectionalLight, DirectionalLightBuffer},
    point_light::{PointLight, PointLightBuffer},
};

static LIGHT_LAYOUT: OnceLock<DescriptorSetLayout> = OnceLock::new();

const DIRECTIONAL_SHADOW_SIZE: u32 = 2048;
const POINT_SHADOW_SIZE: u32 = 1024;
const MAX_CASCADES: u32 = 4;

/// Shadow resource node that manages shadow map textures and samplers
///
/// This node monitors the light count each frame and recreates texture arrays
/// when the count changes. It shares the shadow textures via the render graph
/// context so other passes can access them.
pub struct ShadowResource {
    // Track previous light counts to detect changes
    prev_directional_count: usize,
    prev_point_count: usize,

    // Shadow textures
    directional_shadow_array: Option<TextureArray>,
    point_shadow_cube_array: Option<TextureCubeArray>,

    // Shadow sampler (depth comparison sampler)
    shadow_sampler: Option<Sampler>,

    // Light buffers
    direct_light_buffer: Option<Buffer<DirectionalLightBuffer>>,
    point_light_buffer: Option<Buffer<PointLightBuffer>>,

    // Light descriptor set
    light_descriptor_set: Option<DescriptorSet>,
}

impl Default for ShadowResource {
    fn default() -> Self {
        Self {
            prev_directional_count: 0,
            prev_point_count: 0,
            directional_shadow_array: None,
            point_shadow_cube_array: None,
            shadow_sampler: None,
            direct_light_buffer: None,
            point_light_buffer: None,
            light_descriptor_set: None,
        }
    }
}

impl ShadowResource {
    /// Get or create the shared light descriptor set layout
    pub fn layout(render_ctx: &RenderContext) -> &'static DescriptorSetLayout {
        LIGHT_LAYOUT.get_or_init(|| {
            render_ctx.create_descriptor_set_layout(DescriptorSetLayoutDescriptor {
                label: Some("light layout"),
                visibility: StageFlags::FRAGMENT,
                layout: &[
                    DescriptorBindingType::Storage { read_only: true }, // Binding 0: directional lights
                    DescriptorBindingType::Storage { read_only: true }, // Binding 1: point lights
                    DescriptorBindingType::TextureViewDepthArray, // Binding 2: directional shadow maps
                    DescriptorBindingType::TextureViewDepthCubeArray, // Binding 3: point shadow maps
                    DescriptorBindingType::ComparisonSampler,     // Binding 4: shadow sampler
                ],
            })
        })
    }
}

impl RenderNode for ShadowResource {
    fn setup(
        &mut self,
        render_ctx: &RenderContext,
        _graph_ctx: &mut RenderGraphContext,
    ) -> RenderNodeDescriptor {
        // Create shadow sampler for depth comparison
        let shadow_sampler = render_ctx.create_sampler(SamplerOptions {
            mode_u: TextureMode::ClampToEdge,
            mode_v: TextureMode::ClampToEdge,
            mode_w: TextureMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            compare: Some(maple_renderer::core::DepthCompare::LessEqual),
        });

        self.shadow_sampler = Some(shadow_sampler);

        // Create light buffers
        let direct_light_buffer = render_ctx.create_empty_storage_buffer::<DirectionalLightBuffer>();
        let point_light_buffer = render_ctx.create_empty_storage_buffer::<PointLightBuffer>();

        self.direct_light_buffer = Some(direct_light_buffer);
        self.point_light_buffer = Some(point_light_buffer);

        // Use Marker shader since this node doesn't render anything
        let dummy_shader = render_ctx.create_shader_pair(maple_renderer::core::ShaderPair::Glsl {
            vert: r#"#version 450
void main() {
    gl_Position = vec4(0.0);
}"#,
            frag: r#"#version 450
layout(location = 0) out vec4 outColor;
void main() {
    outColor = vec4(0.0, 0.0, 0.0, 0.0);
}"#,
        });

        RenderNodeDescriptor {
            shader: dummy_shader,
            descriptor_set_layouts: vec![],
            target: vec![],
            depth: DepthTarget::None,
        }
    }

    fn draw(
        &mut self,
        render_ctx: &RenderContext,
        _node_ctx: &mut RenderNodeContext,
        graph_ctx: &mut RenderGraphContext,
        scene: &Scene,
    ) {
        // Count lights in the scene
        let directional_lights = scene.collect_items::<DirectionalLight>();
        let point_lights = scene.collect_items::<PointLight>();

        let directional_count = directional_lights.len();
        let point_count = point_lights.len();

        // Track if we need to rebuild the descriptor set
        let mut rebuild_descriptor = false;

        // Check if we need to recreate directional shadow arrays
        if directional_count != self.prev_directional_count || self.directional_shadow_array.is_none() {
            rebuild_descriptor = true;
            if directional_count != self.prev_directional_count {
                println!(
                    "Directional light count changed: {} -> {}. Recreating shadow maps.",
                    self.prev_directional_count, directional_count
                );
            }

            // Always create at least a minimal 1-layer texture, even with no lights
            let array_layers = if directional_count > 0 {
                (directional_count * MAX_CASCADES as usize)
                    .next_power_of_two()
                    .max(MAX_CASCADES as usize) as u32
            } else {
                1 // Minimal placeholder texture
            };

            let shadow_array = render_ctx.create_texture_array(
                TextureArrayCreateInfo {
                    label: Some("directional_shadows"),
                    width: DIRECTIONAL_SHADOW_SIZE,
                    height: DIRECTIONAL_SHADOW_SIZE,
                    array_layers,
                    format: TextureFormat::Depth32,
                    usage: TextureUsage::RENDER_ATTACHMENT | TextureUsage::TEXTURE_BINDING,
                },
            );

            self.directional_shadow_array = Some(shadow_array);
            self.prev_directional_count = directional_count;
        }

        // Check if we need to recreate point shadow arrays
        if point_count != self.prev_point_count || self.point_shadow_cube_array.is_none() {
            rebuild_descriptor = true;
            if point_count != self.prev_point_count {
                println!(
                    "Point light count changed: {} -> {}. Recreating shadow maps.",
                    self.prev_point_count, point_count
                );
            }

            // Always create at least a minimal 1-layer texture, even with no lights
            let array_layers = if point_count > 0 {
                point_count.next_power_of_two().max(1) as u32
            } else {
                1 // Minimal placeholder texture
            };

            let cube_array = render_ctx.create_texture_cube_array(
                TextureCubeArrayCreateInfo {
                    label: Some("point_shadows"),
                    size: POINT_SHADOW_SIZE,
                    array_layers,
                    format: TextureFormat::Depth32,
                    usage: TextureUsage::RENDER_ATTACHMENT | TextureUsage::TEXTURE_BINDING,
                },
            );

            self.point_shadow_cube_array = Some(cube_array);
            self.prev_point_count = point_count;
        }

        // Rebuild descriptor set if needed (or if it's the first time)
        if rebuild_descriptor || self.light_descriptor_set.is_none() {
            if let (Some(dir_shadows), Some(pt_shadows), Some(sampler), Some(dir_buf), Some(pt_buf)) = (
                &self.directional_shadow_array,
                &self.point_shadow_cube_array,
                &self.shadow_sampler,
                &self.direct_light_buffer,
                &self.point_light_buffer,
            ) {
                let light_layout = Self::layout(render_ctx);
                let light_set = render_ctx.build_descriptor_set(
                    DescriptorSet::builder(light_layout)
                        .storage(0, dir_buf)
                        .storage(1, pt_buf)
                        .texture_view(2, &dir_shadows.create_view())
                        .texture_view(3, &pt_shadows.create_view())
                        .sampler(4, sampler),
                );
                self.light_descriptor_set = Some(light_set);
            }
        }

        // Share resources via graph context
        if let Some(shadow_array) = &self.directional_shadow_array {
            graph_ctx.add_shared_resource("directional_shadows", shadow_array.clone());
        }

        if let Some(cube_array) = &self.point_shadow_cube_array {
            graph_ctx.add_shared_resource("point_shadows", cube_array.clone());
        }

        if let Some(ref sampler) = self.shadow_sampler {
            graph_ctx.add_shared_resource("shadow_sampler", sampler.clone());
        }

        if let Some(ref light_set) = self.light_descriptor_set {
            graph_ctx.add_shared_resource("light_descriptor_set", light_set.clone());
        }

        // Share light buffers so main pass can update them
        if let Some(ref dir_buf) = self.direct_light_buffer {
            graph_ctx.add_shared_resource("direct_light_buffer", dir_buf.clone());
        }

        if let Some(ref pt_buf) = self.point_light_buffer {
            graph_ctx.add_shared_resource("point_light_buffer", pt_buf.clone());
        }
    }
}
