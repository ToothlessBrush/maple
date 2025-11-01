const PI: f32 = 3.14159265359;

struct SceneData {
    background_color: vec4<f32>,
    ambient: f32,
}

struct CameraData {
    cam_pos: vec4<f32>,
    projection: mat4x4<f32>,
    view: mat4x4<f32>,
    VP: mat4x4<f32>,
}

struct MaterialData {
    base_color_factor: vec4<f32>,
    metallic_factor: f32,
    roughness_factor: f32,
    normal_scale: f32,
    ambient_occlusion_strength: f32,
    emissive_factor: vec4<f32>,
    alpha_cutoff: f32,
}

struct MeshData {
    model: mat4x4<f32>,
}

struct DirectLight {
    color: vec4<f32>,
    direction: vec4<f32>,
    intensity: f32,
    shadow_index: i32,
    cascade_level: i32,
    far_plane: f32,
    cascade_split: vec4<f32>,
    light_space_matrices: array<mat4x4<f32>, 4>,
}

struct PointLight {
    color: vec4<f32>,
    pos: vec4<f32>,
    intensity: f32,
    shadow_index: i32,
    far_plane: f32,
    _padding: i32,
}

struct DirectLightBuffer {
    len: i32,
    lights: array<DirectLight>,
}

struct PointLightBuffer {
    len: i32,
    lights: array<PointLight>,
}

@group(0) @binding(0) var<uniform> scene: SceneData;
@group(0) @binding(1) var<uniform> camera: CameraData;
@group(1) @binding(0) var<uniform> material: MaterialData;
@group(2) @binding(0) var<uniform> mesh: MeshData;
@group(3) @binding(0) var<storage, read> direct_light_buffer: DirectLightBuffer;
@group(3) @binding(1) var<storage, read> point_light_buffer: PointLightBuffer;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coord: vec2<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
}

fn distribution_schlick_ggx(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let NdotH2 = NdotH * NdotH;

    let num = a2;
    var denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return num / denom;
}

fn geometry_schlick_ggx(NdotV: f32, roughness: f32) -> f32 {
    let r = (roughness + 1.0);
    let k = (r * r) / 8.0;

    let num = NdotV;
    let denom = NdotV * (1.0 - k) + k;

    return num / denom;
}

fn geometry_smith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let ggx2 = geometry_schlick_ggx(NdotV, roughness);
    let ggx1 = geometry_schlick_ggx(NdotL, roughness);

    return ggx1 * ggx2;
}

fn fresnel_schlick(cosTheta: f32, F0: vec3<f32>) -> vec3<f32> {
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {

    // Base color from material
    let base_color = material.base_color_factor;
    let albedo = pow(base_color.rgb, vec3<f32>(2.2)); // Convert to linear space
    let alpha = base_color.a;

    // Alpha cutoff test
    if alpha < material.alpha_cutoff {
        discard;
    }

    // Material properties
    let metallic = material.metallic_factor;
    let roughness = material.roughness_factor;

    // Normal (no normal mapping for now)
    let N = normalize(in.normal);

    // View direction
    let V = normalize(camera.cam_pos.xyz - in.world_pos);

    // Calculate F0 for PBR
    var F0 = vec3<f32>(0.04);
    F0 = mix(F0, albedo, metallic);

    var Lo = vec3<f32>(0.0);

    // Directional lights
    for (var i: i32 = 0; i < direct_light_buffer.len; i++) {
        let light = direct_light_buffer.lights[i];
        let L = normalize(light.direction.xyz);
        let H = normalize(V + L);

        let NdotL = max(dot(N, L), 0.0);

        let radiance = light.color.rgb * light.intensity;

        // Cook-Torrance BRDF
        let NDF = distribution_schlick_ggx(N, H, roughness);
        let G = geometry_smith(N, V, L, roughness);
        let F = fresnel_schlick(max(dot(H, V), 0.0), F0);

        let numerator = NDF * G * F;
        let denominator = 4.0 * max(dot(N, V), 0.0) * NdotL + 0.0001;
        let specular = numerator / denominator;

        // Energy conservation
        let kS = F;
        let kD = (vec3<f32>(1.0) - kS) * (1.0 - metallic);

        // Add to outgoing radiance
        Lo += (kD * albedo / PI + specular) * radiance * NdotL;
    }

    // Point lights
    for (var i: i32 = 0; i < point_light_buffer.len; i++) {
        let light = point_light_buffer.lights[i];
        let L = normalize(light.pos.xyz - in.world_pos);
        let H = normalize(V + L);

        let light_distance = length(light.pos.xyz - in.world_pos);
        let attenuation = 1.0 / (light_distance * light_distance);

        let radiance = light.color.rgb * attenuation * light.intensity;

        // Cook-Torrance BRDF
        let NDF = distribution_schlick_ggx(N, H, roughness);
        let G = geometry_smith(N, V, L, roughness);
        let F = fresnel_schlick(max(dot(H, V), 0.0), F0);

        let kS = F;
        let kD = (vec3<f32>(1.0) - kS) * (1.0 - metallic);

        let numerator = NDF * G * F;
        let denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
        let specular = numerator / denominator;

        let NdotL = max(dot(N, L), 0.0);

        // Add to outgoing radiance
        Lo += (kD * albedo / PI + specular) * radiance * NdotL;
    }

    // Ambient lighting
    let ambient = vec3<f32>(scene.ambient) * albedo * material.ambient_occlusion_strength;

    // Emissive contribution
    let emissive = material.emissive_factor.xyz;

    // Combine lighting
    var out_color = emissive + ambient + Lo;

    // Tone mapping (Reinhard)
    out_color = out_color / (out_color + vec3<f32>(1.0));

    // Gamma correction
    out_color = pow(out_color, vec3<f32>(1.0 / 2.2));

    return vec4<f32>(out_color.rgb, alpha);
}
