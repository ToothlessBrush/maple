// Directional shadow fragment shader
// Depth is written automatically, no output needed

const ALPHA_MODE_OPAQUE: u32 = 0u;
const ALPHA_MODE_MASK: u32 = 1u;
const ALPHA_MODE_BLEND: u32 = 2u;

struct MaterialData {
    base_alpha_factor: f32,
    alpha_cutoff: f32,
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
    let alpha = base_color.a * material.base_alpha_factor;

    // Handle alpha modes
    if material.alpha_mode == ALPHA_MODE_MASK || material.alpha_mode == ALPHA_MODE_BLEND {
        // Discard fragments below the alpha cutoff
        if alpha < material.alpha_cutoff {
            discard;
        }
    }

    // Depth is written automatically to the depth attachment
    // No fragment output needed for depth-only rendering
}
