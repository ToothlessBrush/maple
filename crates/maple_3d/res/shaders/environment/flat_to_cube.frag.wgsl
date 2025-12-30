@group(0) @binding(0) var equirect_texture: texture_2d<f32>;
@group(0) @binding(1) var equirect_sampler: sampler;

struct Uniforms {
    face_index: u32,
}

@group(0) @binding(2) var<uniform> uniforms: Uniforms;

struct FragmentInput {
    @location(0) local_pos: vec3<f32>,
}

const INV_ATAN: vec2<f32> = vec2<f32>(0.1591, 0.3183);

fn sample_spherical_map(v: vec3<f32>) -> vec2<f32> {
    var uv = vec2<f32>(atan2(v.z, v.x), asin(v.y));
    uv *= INV_ATAN;
    uv += 0.5;
    return uv;
}

fn get_cube_direction(uv: vec2<f32>, face: u32) -> vec3<f32> {
    // Convert UV (-1 to 1) to 3D direction based on cube face
    let u = uv.x;
    let v = uv.y;

    switch face {
        case 0u: { return normalize(vec3<f32>(1.0, -v, -u)); }   // +X
        case 1u: { return normalize(vec3<f32>(-1.0, -v, u)); }   // -X
        case 2u: { return normalize(vec3<f32>(u, 1.0, v)); }     // +Y
        case 3u: { return normalize(vec3<f32>(u, -1.0, -v)); }   // -Y
        case 4u: { return normalize(vec3<f32>(u, -v, 1.0)); }    // +Z
        default: { return normalize(vec3<f32>(-u, -v, -1.0)); }  // -Z
    }
}

@fragment
fn main(in: FragmentInput) -> @location(0) vec4<f32> {
    // Get the direction for this cube face
    let dir = get_cube_direction(in.local_pos.xy, uniforms.face_index);
    
    // Sample equirectangular map
    let uv = sample_spherical_map(dir);
    let color = textureSample(equirect_texture, equirect_sampler, uv).rgb;

    return vec4<f32>(color, 1.0);
}
