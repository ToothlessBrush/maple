// BRDF Integration LUT Generator
// Generates a 2D lookup table for split-sum approximation of specular IBL
// X axis: NdotV (cos angle between normal and view)
// Y axis: roughness

@group(0) @binding(0) var output_texture: texture_storage_2d<rg32float, write>;

const PI: f32 = 3.14159265359;
const SAMPLE_COUNT: u32 = 1024u;

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

fn geometry_schlick_ggx(n_dot_v: f32, roughness: f32) -> f32 {
    let a = roughness;
    let k = (a * a) / 2.0;

    let denom = n_dot_v * (1.0 - k) + k;
    return n_dot_v / denom;
}

fn geometry_smith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let n_dot_v = max(dot(N, V), 0.0);
    let n_dot_l = max(dot(N, L), 0.0);
    let ggx2 = geometry_schlick_ggx(n_dot_v, roughness);
    let ggx1 = geometry_schlick_ggx(n_dot_l, roughness);

    return ggx1 * ggx2;
}

fn integrate_brdf(n_dot_v: f32, roughness: f32) -> vec2<f32> {
    // View vector in tangent space (N pointing straight up)
    var V: vec3<f32>;
    V.x = sqrt(1.0 - n_dot_v * n_dot_v);
    V.y = 0.0;
    V.z = n_dot_v;

    var A = 0.0;
    var B = 0.0;

    let N = vec3<f32>(0.0, 0.0, 1.0);

    for (var i = 0u; i < SAMPLE_COUNT; i++) {
        let xi = hammersley(i, SAMPLE_COUNT);
        let H = importance_sample_ggx(xi, N, roughness);
        let L = normalize(2.0 * dot(V, H) * H - V);

        let n_dot_l = max(L.z, 0.0);
        let n_dot_h = max(H.z, 0.0);
        let v_dot_h = max(dot(V, H), 0.0);

        if n_dot_l > 0.0 {
            let G = geometry_smith(N, V, L, roughness);
            let G_Vis = (G * v_dot_h) / (n_dot_h * n_dot_v);
            let Fc = pow(1.0 - v_dot_h, 5.0);

            A += (1.0 - Fc) * G_Vis;
            B += Fc * G_Vis;
        }
    }

    A /= f32(SAMPLE_COUNT);
    B /= f32(SAMPLE_COUNT);

    return vec2<f32>(A, B);
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
    
    // X axis = NdotV, Y axis = roughness
    let n_dot_v = uv.x;
    let roughness = uv.y;

    let integrated_brdf = integrate_brdf(n_dot_v, roughness);
    
    // Store as vec2 for rg32float format
    textureStore(output_texture, vec2<i32>(coords), vec4<f32>(integrated_brdf, 0.0, 0.0));
}