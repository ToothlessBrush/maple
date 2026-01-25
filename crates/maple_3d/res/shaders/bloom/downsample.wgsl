@group(0) @binding(0) var src_texture: texture_2d<f32>;
@group(0) @binding(1) var src_sampler: sampler;
@group(0) @binding(2) var dst_texture: texture_storage_2d<rgba16float, write>;

struct Uniforms {
    src_resolution: vec2<f32>,
}

@group(0) @binding(3) var<uniform> uniforms: Uniforms;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pixel_coords = vec2<i32>(global_id.xy);
    let dst_size = textureDimensions(dst_texture);

    if pixel_coords.x >= i32(dst_size.x) || pixel_coords.y >= i32(dst_size.y) {
        return;
    }

    let tex_coord = (vec2<f32>(pixel_coords) + 0.5) / vec2<f32>(dst_size);
    let src_coord = tex_coord * uniforms.src_resolution;  // In pixels 

    // Take 13 samples around current texel:
    // a - b - c
    // - j - k -
    // d - e - f
    // - l - m -
    // g - h - i
    // === ('e' is the current texel) ===
    let a = textureLoad(src_texture, vec2<i32>(src_coord + vec2<f32>(-2.0, 2.0)), 0).rgb;
    let b = textureLoad(src_texture, vec2<i32>(src_coord + vec2<f32>(0.0, 2.0)), 0).rgb;
    let c = textureLoad(src_texture, vec2<i32>(src_coord + vec2<f32>(2.0, 2.0)), 0).rgb;

    let d = textureLoad(src_texture, vec2<i32>(src_coord + vec2<f32>(-2.0, 0.0)), 0).rgb;
    let e = textureLoad(src_texture, vec2<i32>(src_coord + vec2<f32>(0.0, 0.0)), 0).rgb;
    let f = textureLoad(src_texture, vec2<i32>(src_coord + vec2<f32>(2.0, 0.0)), 0).rgb;

    let g = textureLoad(src_texture, vec2<i32>(src_coord + vec2<f32>(-2.0, -2.0)), 0).rgb;
    let h = textureLoad(src_texture, vec2<i32>(src_coord + vec2<f32>(0.0, -2.0)), 0).rgb;
    let i = textureLoad(src_texture, vec2<i32>(src_coord + vec2<f32>(2.0, -2.0)), 0).rgb;

    let j = textureLoad(src_texture, vec2<i32>(src_coord + vec2<f32>(-1.0, 1.0)), 0).rgb;
    let k = textureLoad(src_texture, vec2<i32>(src_coord + vec2<f32>(1.0, 1.0)), 0).rgb;
    let l = textureLoad(src_texture, vec2<i32>(src_coord + vec2<f32>(-1.0, -1.0)), 0).rgb;
    let m = textureLoad(src_texture, vec2<i32>(src_coord + vec2<f32>(1.0, -1.0)), 0).rgb;

    // Apply weighted distribution:
    // 0.5 + 0.125 + 0.125 + 0.125 + 0.125 = 1
    // a,b,d,e * 0.125
    // b,c,e,f * 0.125
    // d,e,g,h * 0.125
    // e,f,h,i * 0.125
    // j,k,l,m * 0.5
    var downsample = e * 0.125;
    downsample += (a + c + g + i) * 0.03125;
    downsample += (b + d + f + h) * 0.0625;
    downsample += (j + k + l + m) * 0.125;

    textureStore(dst_texture, pixel_coords, vec4<f32>(downsample, 1.0));
}
