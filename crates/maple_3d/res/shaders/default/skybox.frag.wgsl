@group(1) @binding(0) var environment_map: texture_cube<f32>;
@group(1) @binding(1) var environment_sampler: sampler;

struct FragmentInput {
    @location(0) local_pos: vec3<f32>,
}

@fragment
fn main(input: FragmentInput) -> @location(0) vec4<f32> {
    // DEBUG: visualize the sampling direction as colors
    return vec4<f32>(normalize(input.local_pos) * 0.5 + 0.5, 1.0);

    // Normalize to get correct sampling direction (try without negation first)
    var sample_dir = normalize(input.local_pos);
    var env_color = textureSample(environment_map, environment_sampler, sample_dir).rgb;

    // Tone mapping (Reinhard)
    env_color = env_color / (env_color + vec3<f32>(1.0));

    // Gamma correction
    env_color = pow(env_color, vec3<f32>(1.0 / 2.2));

    return vec4<f32>(env_color, 1.0);
}
