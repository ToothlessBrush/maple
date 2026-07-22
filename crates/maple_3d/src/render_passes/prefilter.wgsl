// Prefilter environment map for specular IBL using importance sampling
struct Uniforms {
    roughness: f32,
    face: u32,        // Which cubemap face (0-5)
    mip_level: u32,   // Which mip level to generate
    resolution: f32,  // Resolution of source cubemap (per face)
}

@group(0) @binding(0) var environment_map: texture_cube<f32>;
@group(0) @binding(1) var env_sampler: sampler;
@group(0) @binding(2) var output_texture: texture_storage_2d<rgba16float, write>;
@group(0) @binding(3) var<uniform> uniforms: Uniforms;

const PI: f32 = 3.14159265359;

fn radical_inverse_vdc(bits_in: u32) -> f32 {
    var bits = bits_in;
    bits = (bits << 16u) | (bits >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    return f32(bits) * 2.3283064365386963e-10;
}

fn hammersley(i: u32, N: u32) -> vec2<f32> {
    return vec2<f32>(f32(i) / f32(N), radical_inverse_vdc(i));
}

fn importance_sample_ggx(xi: vec2<f32>, N: vec3<f32>, roughness: f32) -> vec3<f32> {
    let a = roughness * roughness;
    let phi = 2.0 * PI * xi.x;
    let cos_theta = sqrt((1.0 - xi.y) / (1.0 + (a * a - 1.0) * xi.y));
    let sin_theta = sqrt(1.0 - cos_theta * cos_theta);
    
    // Spherical to cartesian coordinates
    var H: vec3<f32>;
    H.x = cos(phi) * sin_theta;
    H.y = sin(phi) * sin_theta;
    H.z = cos_theta;
    
    // Tangent space to world space
    let up = select(vec3<f32>(1.0, 0.0, 0.0), vec3<f32>(0.0, 0.0, 1.0), abs(N.z) < 0.999);
    let tangent = normalize(cross(up, N));
    let bitangent = cross(N, tangent);

    let sample_vec = tangent * H.x + bitangent * H.y + N * H.z;
    return normalize(sample_vec);
}

fn distribution_ggx(NdotH: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let denom = NdotH * NdotH * (a2 - 1.0) + 1.0;
    return a2 / (PI * denom * denom);
}

// Convert UV coordinates and face index to 3D direction
fn uv_to_direction(uv: vec2<f32>, face: u32) -> vec3<f32> {
    let u = uv.x * 2.0 - 1.0;
    let v = uv.y * 2.0 - 1.0;

    switch face {
        case 0u: { return normalize(vec3<f32>(1.0, -v, -u)); }   // +X
        case 1u: { return normalize(vec3<f32>(-1.0, -v, u)); }   // -X
        case 2u: { return normalize(vec3<f32>(u, 1.0, v)); }     // +Y
        case 3u: { return normalize(vec3<f32>(u, -1.0, -v)); }   // -Y
        case 4u: { return normalize(vec3<f32>(u, -v, 1.0)); }    // +Z
        default: { return normalize(vec3<f32>(-u, -v, -1.0)); }  // -Z
    }
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let dims = textureDimensions(output_texture);
    let coords = global_id.xy;

    if coords.x >= dims.x || coords.y >= dims.y {
        return;
    }
    
    // Convert pixel coordinates to UV [0, 1]
    let uv = (vec2<f32>(coords) + 0.5) / vec2<f32>(dims);
    
    // Get the direction for this pixel
    let N = uv_to_direction(uv, uniforms.face);
    let R = N;
    let V = R;

    var prefiltered_color = vec3<f32>(0.0);
    var total_weight = 0.0;

    // bigger number is better quality but slower to load
    let base_samples = 4096u;
    let sample_multiplier = 1.0 + uniforms.roughness * 3.0;
    let sample_count = u32(f32(base_samples) * sample_multiplier);
    
    // Precompute solid angle of a texel
    let sa_texel = 4.0 * PI / (6.0 * uniforms.resolution * uniforms.resolution);

    for (var i = 0u; i < sample_count; i++) {
        let xi = hammersley(i, sample_count);
        let H = importance_sample_ggx(xi, N, uniforms.roughness);
        let L = normalize(2.0 * dot(V, H) * H - V);

        let n_dot_l = max(dot(N, L), 0.0);

        if n_dot_l > 0.0 {
            let n_dot_h = max(dot(N, H), 0.0);
            let h_dot_v = max(dot(H, V), 0.0);
            
            // Calculate PDF and mip level for this sample
            let D = distribution_ggx(n_dot_h, uniforms.roughness);
            let pdf = (D * n_dot_h / (4.0 * h_dot_v)) + 0.0001;

            let sa_sample = 1.0 / (f32(sample_count) * pdf + 0.0001);
            
            // Clamp mip level to reasonable range
            var mip_level = 0.0;
            if uniforms.roughness > 0.0 {
                mip_level = clamp(0.5 * log2(sa_sample / sa_texel), 0.0, 10.0);
            }

            prefiltered_color += textureSampleLevel(environment_map, env_sampler, L, mip_level).rgb * n_dot_l;
            total_weight += n_dot_l;
        }
    }

    // Avoid division by zero
    if total_weight > 0.0 {
        prefiltered_color = prefiltered_color / total_weight;
    }

    textureStore(output_texture, vec2<i32>(coords), vec4<f32>(prefiltered_color, 1.0));
}