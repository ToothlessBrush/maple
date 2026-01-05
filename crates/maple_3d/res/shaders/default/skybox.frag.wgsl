@group(1) @binding(0) var environment_map: texture_cube<f32>;
@group(1) @binding(1) var environment_sampler: sampler;

struct FragmentInput {
    @location(0) local_pos: vec3<f32>,
}

@fragment
fn main(input: FragmentInput) -> @location(0) vec4<f32> {
    // Normalize to get correct sampling direction (try without negation first)
    var sample_dir = normalize(input.local_pos);
    var env_color = textureSample(environment_map, environment_sampler, sample_dir).rgb;

    // Tone mapping (Reinhard)

    // Gamma correction

    return vec4<f32>(env_color, 1.0);
}
