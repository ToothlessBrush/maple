// Directional shadow fragment shader
// Depth is written automatically, no output needed

const ALPHA_MODE_OPAQUE: u32 = 0u;
const ALPHA_MODE_MASK: u32 = 1u;
const ALPHA_MODE_BLEND: u32 = 2u;

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

@group(2) @binding(0) var<uniform> material: MaterialData;
@group(2) @binding(1) var base_color_texture: texture_2d<f32>;
@group(2) @binding(2) var base_color_sampler: sampler;

struct FragmentInput {
    @location(0) tex_coord: vec2<f32>,
}

@fragment
fn main(input: FragmentInput) {
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

    // Depth is written automatically to the depth attachment
    // No fragment output needed for depth-only rendering
}
