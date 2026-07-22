@group(0) @binding(0) var src_texture: texture_2d<f32>;
@group(0) @binding(1) var src_sampler: sampler;

struct Uniforms {
    filter_radius: f32,
}

@group(0) @binding(2) var<uniform> uniforms: Uniforms;

@fragment
fn main(@location(0) tex_coord: vec2<f32>) -> @location(0) vec4<f32> {
    // The filter kernel is applied with a radius, specified in texture
    // coordinates, so that the radius will vary across mip resolutions.
    let x = uniforms.filter_radius;
    let y = uniforms.filter_radius;

    // Take 9 samples around current texel:
    // a - b - c
    // d - e - f
    // g - h - i
    // === ('e' is the current texel) ===
    let a = textureSample(src_texture, src_sampler, vec2<f32>(tex_coord.x - x, tex_coord.y + y)).rgb;
    let b = textureSample(src_texture, src_sampler, vec2<f32>(tex_coord.x, tex_coord.y + y)).rgb;
    let c = textureSample(src_texture, src_sampler, vec2<f32>(tex_coord.x + x, tex_coord.y + y)).rgb;

    let d = textureSample(src_texture, src_sampler, vec2<f32>(tex_coord.x - x, tex_coord.y)).rgb;
    let e = textureSample(src_texture, src_sampler, vec2<f32>(tex_coord.x, tex_coord.y)).rgb;
    let f = textureSample(src_texture, src_sampler, vec2<f32>(tex_coord.x + x, tex_coord.y)).rgb;

    let g = textureSample(src_texture, src_sampler, vec2<f32>(tex_coord.x - x, tex_coord.y - y)).rgb;
    let h = textureSample(src_texture, src_sampler, vec2<f32>(tex_coord.x, tex_coord.y - y)).rgb;
    let i = textureSample(src_texture, src_sampler, vec2<f32>(tex_coord.x + x, tex_coord.y - y)).rgb;

    // Apply weighted distribution, by using a 3x3 tent filter:
    //  1   | 1 2 1 |
    // -- * | 2 4 2 |
    // 16   | 1 2 1 |
    var upsample = e * 4.0;
    upsample += (b + d + f + h) * 2.0;
    upsample += (a + c + g + i);
    upsample *= 1.0 / 16.0;

    return vec4<f32>(upsample, 1.0);
}
