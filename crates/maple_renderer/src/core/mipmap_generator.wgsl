
// Mipmap generation compute shader
// Downsamples 2x2 texels from source mip level to 1 texel in destination mip level

@group(0) @binding(0) var src_texture: texture_2d<f32>;
@group(0) @binding(1) var dst_texture: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(2) var src_sampler: sampler;

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let dst_size = textureDimensions(dst_texture);

    // Check bounds
    if global_id.x >= dst_size.x || global_id.y >= dst_size.y {
        return;
    }

    let src_size = textureDimensions(src_texture);

    // Calculate source texel coordinates (2x destination coordinates)
    let src_x = global_id.x * 2u;
    let src_y = global_id.y * 2u;

    // Manual 2x2 box filter (works for both filterable and non-filterable formats)
    var color = vec4<f32>(0.0);
    var count = 0.0;

    // Sample 2x2 neighborhood
    for (var dy = 0u; dy < 2u; dy = dy + 1u) {
        for (var dx = 0u; dx < 2u; dx = dx + 1u) {
            let sample_x = min(src_x + dx, src_size.x - 1u);
            let sample_y = min(src_y + dy, src_size.y - 1u);
            color += textureLoad(src_texture, vec2<i32>(i32(sample_x), i32(sample_y)), 0);
            count += 1.0;
        }
    }

    // Average the samples
    color /= count;

    // Write to destination
    textureStore(dst_texture, vec2<i32>(global_id.xy), color);
}
