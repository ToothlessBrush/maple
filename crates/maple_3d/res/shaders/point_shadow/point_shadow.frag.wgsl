// Point shadow fragment shader
// Calculate and store distance from light for point light shadows

const ALPHA_MODE_OPAQUE: u32 = 0u;
const ALPHA_MODE_MASK: u32 = 1u;
const ALPHA_MODE_BLEND: u32 = 2u;

struct LightData {
    view_projection: mat4x4<f32>,
    light_pos: vec4<f32>,
    far_plane: f32,
    _padding: vec3<f32>,
}

struct MaterialData {
    base_color_factor: vec4<f32>,
    metallic_factor: f32,
    roughness_factor: f32,
    normal_scale: f32,
    ambient_occlusion_strength: f32,
    emissive_factor: vec4<f32>,
    alpha_cutoff: f32,
    parallax_scale: f32,
    alpha_mode: u32,
}

@group(0) @binding(0) var<uniform> light: LightData;

@group(2) @binding(0) var<uniform> material: MaterialData;
@group(2) @binding(1) var base_color_texture: texture_2d<f32>;
@group(2) @binding(2) var base_color_sampler: sampler;

struct FragmentInput {
    @builtin(position) frag_coord: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
}

struct FragmentOutput {
    @builtin(frag_depth) depth: f32,
}

@fragment
fn main(input: FragmentInput) -> FragmentOutput {
    // Sample base color to get alpha
    let base_color = textureSample(base_color_texture, base_color_sampler, input.tex_coord);
    let alpha = base_color.a * material.base_color_factor.a;

    // Handle alpha modes
    if material.alpha_mode == ALPHA_MODE_MASK {
        // Discard fragments below the alpha cutoff
        if alpha < material.alpha_cutoff {
            discard;
        }
    } else if material.alpha_mode == ALPHA_MODE_BLEND {
        // Discard fully or nearly transparent fragments
        if alpha < 0.5 {
            discard;
        }
    }

    // Store the linear distance from light to fragment
    // This needs to match what we compare in the main shader
    let distance = length(input.world_pos - light.light_pos.xyz);
    let normalized_distance = distance / light.far_plane;

    var output: FragmentOutput;
    output.depth = normalized_distance;
    return output;
}
