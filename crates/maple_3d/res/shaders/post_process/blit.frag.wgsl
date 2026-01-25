@group(0) @binding(0) var scene_texture: texture_2d<f32>;
@group(0) @binding(1) var bloom_texture: texture_2d<f32>;
@group(0) @binding(2) var tex_sampler: sampler;

struct Uniforms {
    bloom_intensity: f32,
    exposure: f32,
    _padding: vec2<f32>,
}

@group(0) @binding(3) var<uniform> uniforms: Uniforms;

// ACES fitted curve (Krzysztof Narkowicz approximation)
fn aces_tonemap(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return saturate((x * (a * x + b)) / (x * (c * x + d) + e));
}


@fragment
fn main(@location(0) tex_coord: vec2<f32>) -> @location(0) vec4<f32> {
    let scene = textureSample(scene_texture, tex_sampler, tex_coord).rgb;
    let bloom = textureSample(bloom_texture, tex_sampler, tex_coord).rgb;

    var hdr = scene + bloom * uniforms.bloom_intensity;

    // Apply exposure before tonemapping
    hdr = hdr * uniforms.exposure;

    let ldr = aces_tonemap(hdr);

    return vec4<f32>(ldr, 1.0);
}
