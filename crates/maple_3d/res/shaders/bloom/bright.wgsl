@group(0) @binding(0) var src_texture: texture_2d<f32>;
@group(0) @binding(1) var dst_texture: texture_storage_2d<rgba16float, write>;

// ACES fitted curve (Krzysztof Narkowicz approximation)                                                                           
fn aces_tonemap(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return saturate((x * (a * x + b)) / (x * (c * x + d) + e));
} 

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let coords = vec2<i32>(global_id.xy);
    let size = textureDimensions(dst_texture);

    if coords.x >= i32(size.x) || coords.y >= i32(size.y) {
        return;
    }

    let color = textureLoad(src_texture, coords, 0).rgb;
    let brightness = max(color.r, max(color.g, color.b));
    let contribution = max(brightness - 1.0, 0.0);
    let result = color * (contribution / max(brightness, 0.0001));

    textureStore(dst_texture, coords, vec4<f32>(result, 1.0));
}
