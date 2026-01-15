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
    render_graph::{graph::RenderGraphContext, node::RenderNode},
};

use crate::nodes::{
    directional_light::{DirectionalLight, DirectionalLightBuffer},
    point_light::{PointLight, PointLightBuffer},
};

const DIRECTIONAL_SHADOW_SIZE: u32 = 4096;
const POINT_SHADOW_SIZE: u32 = 1024;
const MAX_CASCADES: u32 = 4;

/// Shadow resource node that manages shadow map textures and samplers
///
/// This node monitors the light count each frame and recreates texture arrays
/// when the count changes. It shares the shadow textures via the render graph
/// context so other passes can access them.
struct ShadowTextureSet {
    directional_shadow_array: TextureArray,
    point_shadow_cube_array: TextureCubeArray,
    shadow_sampler: Sampler,
    direct_light_buffer: Buffer<DirectionalLightBuffer>,
    point_light_buffer: Buffer<PointLightBuffer>,
    light_descriptor_set: DescriptorSet,
}

impl ShadowTextureSet {
    fn create(rcx: &RenderContext, directional_count: usize, point_count: usize) -> Self {
        // Create shadow sampler for depth comparison
        let shadow_sampler = rcx.create_sampler(SamplerOptions {
            mode_u: TextureMode::ClampToEdge,
            mode_v: TextureMode::ClampToEdge,
            mode_w: TextureMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            compare: Some(maple_renderer::core::DepthCompare::LessEqual),
        });

        // Create light buffers
        let direct_light_buffer = rcx.create_empty_storage_buffer::<DirectionalLightBuffer>();
        let point_light_buffer = rcx.create_empty_storage_buffer::<PointLightBuffer>();

        // Create directional shadow array (always at least 1 layer)
        let dir_array_layers = if directional_count > 0 {
            (directional_count * MAX_CASCADES as usize)
                .next_power_of_two()
                .max(MAX_CASCADES as usize) as u32
        } else {
            1
        };

        let directional_shadow_array = rcx.create_texture_array(TextureArrayCreateInfo {
            label: Some("directional_shadows"),
            width: DIRECTIONAL_SHADOW_SIZE,
            height: DIRECTIONAL_SHADOW_SIZE,
            array_layers: dir_array_layers,
            format: TextureFormat::Depth32,
            usage: TextureUsage::RENDER_ATTACHMENT | TextureUsage::TEXTURE_BINDING,
        });

        // Create point shadow cube array (always at least 1 layer)
        let point_array_layers = if point_count > 0 {
            point_count.next_power_of_two().max(1) as u32
        } else {
            1
        };

        let point_shadow_cube_array = rcx.create_texture_cube_array(TextureCubeArrayCreateInfo {
            label: Some("point_shadows"),
            size: POINT_SHADOW_SIZE,
            array_layers: point_array_layers,
            format: TextureFormat::Depth32,
            usage: TextureUsage::RENDER_ATTACHMENT | TextureUsage::TEXTURE_BINDING,
        });

        // Build descriptor set
        let light_layout = ShadowResource::layout(rcx);
        let light_descriptor_set = rcx.build_descriptor_set(
            DescriptorSet::builder(&light_layout)
                .storage(0, &direct_light_buffer)
                .storage(1, &point_light_buffer)
                .texture_view(2, &directional_shadow_array.create_view())
                .texture_view(3, &point_shadow_cube_array.create_view())
                .sampler(4, &shadow_sampler),
        );

        Self {
            directional_shadow_array,
            point_shadow_cube_array,
            shadow_sampler,
            direct_light_buffer,
            point_light_buffer,
            light_descriptor_set,
        }
    }

    fn share_to_graph(&self, gcx: &mut RenderGraphContext) {
        gcx.add_shared_resource("directional_shadows", self.directional_shadow_array.clone());
        gcx.add_shared_resource("point_shadows", self.point_shadow_cube_array.clone());
        gcx.add_shared_resource("shadow_sampler", self.shadow_sampler.clone());
        gcx.add_shared_resource("direct_light_buffer", self.direct_light_buffer.clone());
        gcx.add_shared_resource("point_light_buffer", self.point_light_buffer.clone());
        gcx.add_shared_resource("light_descriptor_set", self.light_descriptor_set.clone());
    }
}

pub struct ShadowResource {
    textures: ShadowTextureSet,
    prev_directional_count: usize,
    prev_point_count: usize,
}

impl ShadowResource {
    /// Get or create the shared light descriptor set layout
    pub fn layout(rcx: &RenderContext) -> DescriptorSetLayout {
        rcx.get_or_create_layout(
            "light",
            DescriptorSetLayoutDescriptor {
                label: Some("light layout"),
                visibility: StageFlags::FRAGMENT,
                layout: &[
                    DescriptorBindingType::Storage { read_only: true }, // Binding 0: directional lights
                    DescriptorBindingType::Storage { read_only: true }, // Binding 1: point lights
                    DescriptorBindingType::TextureViewDepthArray, // Binding 2: directional shadow maps
                    DescriptorBindingType::TextureViewDepthCubeArray, // Binding 3: point shadow maps
                    DescriptorBindingType::ComparisonSampler,         // Binding 4: shadow sampler
                ],
            },
        )
    }

    pub fn setup(rcx: &RenderContext, gcx: &mut RenderGraphContext) -> Self {
        // Create initial resources with 0 lights
        let textures = ShadowTextureSet::create(rcx, 0, 0);
        textures.share_to_graph(gcx);

        Self {
            textures,
            prev_directional_count: 0,
            prev_point_count: 0,
        }
    }
}

impl RenderNode for ShadowResource {
    fn draw(&mut self, rcx: &RenderContext, gcx: &mut RenderGraphContext, scene: &Scene) {
        // Count lights in the scene
        let directional_lights = scene.collect::<DirectionalLight>();
        let point_lights = scene.collect::<PointLight>();

        let directional_count = directional_lights.len();
        let point_count = point_lights.len();

        // Check if light counts changed - recreate if needed
        if directional_count != self.prev_directional_count || point_count != self.prev_point_count
        {
            if directional_count != self.prev_directional_count {
                log::info!(
                    "Directional light count changed: {} -> {}. Recreating shadow maps.",
                    self.prev_directional_count,
                    directional_count
                );
            }

            if point_count != self.prev_point_count {
                log::info!(
                    "Point light count changed: {} -> {}. Recreating shadow maps.",
                    self.prev_point_count,
                    point_count
                );
            }

            // Recreate entire texture set with new light counts
            self.textures = ShadowTextureSet::create(rcx, directional_count, point_count);
            self.prev_directional_count = directional_count;
            self.prev_point_count = point_count;
        }

        // Re-share resources (they might have been recreated)
        self.textures.share_to_graph(gcx);
    }
}
