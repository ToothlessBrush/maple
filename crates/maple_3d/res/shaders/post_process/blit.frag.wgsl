@group(0) @binding(0) var color_texture: texture_2d<f32>;
@group(0) @binding(1) var color_sampler: sampler;

@fragment
fn main(@location(0) tex_coord: vec2<f32>) -> @location(0) vec4<f32> {
    let hdr_color = textureSample(color_texture, color_sampler, tex_coord).rgb;

    let ldr_color = hdr_color / (hdr_color + vec3<f32>(1.0));

    return vec4<f32>(ldr_color, 1.0);
}
