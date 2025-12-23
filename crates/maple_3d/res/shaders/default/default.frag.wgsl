const PI: f32 = 3.14159265359;

struct SceneData {
    background_color: vec4<f32>,
    ambient: f32,
}

struct CameraData {
    cam_pos: vec4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    VP: mat4x4<f32>,
    far_plane: f32
}

struct MaterialData {
    base_color_factor: vec4<f32>,
    metallic_factor: f32,
    roughness_factor: f32,
    normal_scale: f32,
    ambient_occlusion_strength: f32,
    emissive_factor: vec4<f32>,
    alpha_cutoff: f32,
    parallax_scale: f32,
    use_alpha_mask: f32,
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
    bias: f32,
    cascade_split: vec4<f32>,
    light_space_matrices: array<mat4x4<f32>, 4>,
}

struct PointLight {
    color: vec4<f32>,
    pos: vec4<f32>,
    intensity: f32,
    shadow_index: i32,
    far_plane: f32,
    bias: f32,
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
@group(1) @binding(1) var base_color_texture: texture_2d<f32>;
@group(1) @binding(2) var base_color_sampler: sampler;
@group(1) @binding(3) var metallic_roughness_texture: texture_2d<f32>;
@group(1) @binding(4) var metallic_roughness_sampler: sampler;
@group(1) @binding(5) var ambient_occlusion_texture: texture_2d<f32>;
@group(1) @binding(6) var ambient_occlusion_sampler: sampler;
@group(1) @binding(7) var emissive_texture: texture_2d<f32>;
@group(1) @binding(8) var emissive_sampler: sampler;
@group(1) @binding(9) var normal_texture: texture_2d<f32>;
@group(1) @binding(10) var normal_sampler: sampler;
@group(1) @binding(11) var parallax_texture: texture_2d<f32>;
@group(1) @binding(12) var parallax_sampler: sampler;

@group(2) @binding(0) var<uniform> mesh: MeshData;

@group(3) @binding(0) var<storage, read> direct_light_buffer: DirectLightBuffer;
@group(3) @binding(1) var<storage, read> point_light_buffer: PointLightBuffer;
@group(3) @binding(2) var directional_shadow_maps: texture_depth_2d_array;
@group(3) @binding(3) var point_shadow_maps: texture_depth_cube_array;
@group(3) @binding(4) var shadow_sampler: sampler_comparison;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_coord: vec2<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
    @location(5) tangent_view_pos: vec3<f32>,
    @location(6) tangent_frag_pos: vec3<f32>,
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

fn get_cascade_data(light: DirectLight, cascade_index: i32) -> mat4x4<f32> {
    switch cascade_index {
          case 0: { return light.light_space_matrices[0]; }
          case 1: { return light.light_space_matrices[1]; }
          case 2: { return light.light_space_matrices[2]; }
          default: { return light.light_space_matrices[3]; }
      }
}

fn get_cascade_split(light: DirectLight, cascade_index: i32) -> f32 {
    switch cascade_index {
          case 0: { return light.cascade_split[0]; }
          case 1: { return light.cascade_split[1]; }
          case 2: { return light.cascade_split[2]; }
          default: { return light.cascade_split[3]; }
      }
}

// sample a cascade texture
fn sample_cascade_shadow(
    light: DirectLight,
    world_pos: vec3<f32>,
    normal: vec3<f32>,
    cascade_index: i32
) -> f32 {
    // Transform to light space
    // Get the light space matrix based on cascade index
    var light_space_matrix = get_cascade_data(light, cascade_index);
    var cascade_split_value = get_cascade_split(light, cascade_index);

    // this gets around the stupid cant dynamically index arrays rule


    let light_space_pos = light_space_matrix * vec4<f32>(world_pos, 1.0);
    var proj_coords = light_space_pos.xyz / light_space_pos.w;

    // Transform XY to [0, 1] range for texture sampling (Z is already in that range)
    proj_coords.x = proj_coords.x * 0.5 + 0.5;
    proj_coords.y = proj_coords.y * 0.5 + 0.5;
    // Flip Y for WebGPU texture coordinates (origin at top-left)
    proj_coords.y = 1.0 - proj_coords.y;

    // Check if position is outside shadow map bounds
    if proj_coords.x < 0.0 || proj_coords.x > 1.0 || proj_coords.y < 0.0 || proj_coords.y > 1.0 || proj_coords.z > 1.0 {
        return 1.0;
    }

    // Calculate slope-based bias (based on angle between normal and light)
    let light_dir = normalize(-light.direction.xyz);
    let base_bias = max(light.bias * (1.0 - dot(normal, light_dir)), light.bias);

    // Scale bias inversely with cascade distance to prevent peter panning in far cascades
    // Farther cascades need less bias since they cover larger world areas
    var final_bias: f32;
    if cascade_index == light.cascade_level - 1 {
        // Last cascade uses far plane
        final_bias = base_bias * (1.0 / (camera.far_plane * 0.5));
    } else {
        // Other cascades use their split distance
        final_bias = base_bias * (1.0 / (light.cascade_split[cascade_index] * 0.5));
    }

    let biased_depth = proj_coords.z - final_bias;

    // Calculate shadow map array index
    let shadow_layer = light.shadow_index * 4 + cascade_index;


    // PCF 
    let shadow_map_dim = textureDimensions(directional_shadow_maps);
    let texel_size = 1.0 / vec2<f32>(shadow_map_dim);

    var shadow = 0.0;

    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let offset = vec2<f32>(f32(x), f32(y)) * texel_size;
            shadow += textureSampleCompareLevel(
                directional_shadow_maps,
                shadow_sampler,
                proj_coords.xy + offset,
                shadow_layer,
                biased_depth
            );
        }
    }
    return shadow / 9.0;
}

// Calculate shadow factor for directional lights with cascade blending
fn calculate_directional_shadow(light: DirectLight, world_pos: vec3<f32>, normal: vec3<f32>) -> f32 {
    if light.shadow_index < 0 {
        return 1.0; // No shadow
    }

    // Select cascade based on depth
    let view_pos = camera.view * vec4<f32>(world_pos, 1.0);
    let depth = abs(view_pos.z);

    var cascade_index = light.cascade_level - 1; // Default to furthest cascade
    var blend_factor = 0.0;

    // Find which cascade we're in and calculate blend factor
    for (var i = 0; i < light.cascade_level; i++) {
        if depth < light.cascade_split[i] {
            cascade_index = i;

            // Calculate blend factor for smooth transitions
            // Blend in the last 10% of the cascade range
            if i > 0 {
                let prev_split = select(0.0, light.cascade_split[i - 1], i > 0);
                let cascade_range = light.cascade_split[i] - prev_split;
                let blend_range = cascade_range * 0.1; // 10% transition zone
                let distance_to_end = light.cascade_split[i] - depth;

                if distance_to_end < blend_range {
                    blend_factor = 1.0 - (distance_to_end / blend_range);
                }
            }
            break;
        }
    }

    // Sample current cascade
    let shadow = sample_cascade_shadow(light, world_pos, normal, cascade_index);

    // Blend with next cascade if in transition zone
    if blend_factor > 0.0 && cascade_index < light.cascade_level - 1 {
        let next_shadow = sample_cascade_shadow(light, world_pos, normal, cascade_index + 1);
        return mix(shadow, next_shadow, blend_factor);
    }

    return shadow;
}

fn calculate_point_shadow(light: PointLight, world_pos: vec3<f32>) -> f32 {
    if light.shadow_index < 0 {
        return 1.0; // No shadow
    }
    
    // Get vector from light to fragment
    let light_to_frag = world_pos - light.pos.xyz;
    
    // Calculate current depth and normalize it
    let current_depth = length(light_to_frag);
    let normalized_depth = current_depth / light.far_plane;
    
    // Early exit if out of range
    if normalized_depth > 1.0 {
        return 1.0; // Beyond shadow range
    }
    
    // flip Y
    let sample_dir = light_to_frag * vec3<f32>(1.0, -1.0, 1.0);
    
    // Apply bias to prevent shadow acne
    let compare_depth = saturate(normalized_depth - light.bias);
    
    // Sample shadow cube map
    let shadow = textureSampleCompare(
        point_shadow_maps,
        shadow_sampler,
        sample_dir,
        light.shadow_index,
        compare_depth
    );

    return shadow;
}

fn parallax_mapping(tex_coords: vec2<f32>, view_dir: vec3<f32>) -> vec2<f32> {
    // Number of depth layers
    let min_layers = 8.0;
    let max_layers = 32.0;
    let num_layers = mix(max_layers, min_layers, max(dot(vec3(0.0, 0.0, 1.0), view_dir), 0.0));

    // Calculate the size of each layer
    let layer_depth = 1.0 / num_layers;
    // Depth of current layer
    var current_layer_depth = 0.0;
    // The amount to shift the texture coordinates per layer (from vector P)
    let P = view_dir.xy * material.parallax_scale; // assuming you have this in your material
    let delta_tex_coords = P / num_layers;
    
    // Get initial values
    var current_tex_coords = tex_coords;
    var current_depth_map_value = textureSample(parallax_texture, parallax_sampler, current_tex_coords).r;
    
    // Iterate until we find a depth value less than the layer's depth
    while current_layer_depth < current_depth_map_value {
        // Shift texture coordinates along direction of P
        current_tex_coords -= delta_tex_coords;
        // Get depthmap value at current texture coordinates
        current_depth_map_value = textureSample(parallax_texture, parallax_sampler, current_tex_coords).r;
        // Get depth of next layer
        current_layer_depth += layer_depth;
    }

    let prev_tex_coords = current_tex_coords + delta_tex_coords;

    let after_depth = current_depth_map_value - current_layer_depth;
    let before_depth = textureSample(parallax_texture, parallax_sampler, prev_tex_coords).r - current_layer_depth + layer_depth;

    let weight = after_depth / (after_depth - before_depth);
    let final_tex_coords = prev_tex_coords * weight + current_tex_coords * (1.0 - weight);

    return final_tex_coords;
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(1) normal: vec4<f32>,
}

@fragment
fn main(in: VertexOutput) -> FragmentOutput {

    // View direction in tangent space
    let V = normalize(in.tangent_view_pos - in.tangent_frag_pos);
    // let tex_coords = parallax_mapping(in.tex_coord, V);

    // if tex_coords.x > 1.0 || tex_coords.y > 1.0 || tex_coords.x < 0.0 || tex_coords.y < 0.0 {
    //     discard;
    // }

    let tex_coords = in.tex_coord;

    // Base color from material
    let base_color = textureSample(base_color_texture, base_color_sampler, tex_coords) * material.base_color_factor;
    let albedo = pow(base_color.rgb, vec3<f32>(2.2)); // Convert to linear space
    var alpha = base_color.a;

    // Alpha cutoff test (only when alpha mode is MASK)
    if material.use_alpha_mask > 0.5 && alpha < material.alpha_cutoff {
        discard;
    }

    if material.use_alpha_mask < 0.5 {
        alpha = 1.0;
    }

    let metallic_roughness = textureSample(
        metallic_roughness_texture,
        metallic_roughness_sampler,
        tex_coords
    );

    // Material properties
    let metallic = metallic_roughness.b * material.metallic_factor;
    let roughness = metallic_roughness.g * material.roughness_factor;

    // Normals (glTF uses OpenGL convention with Y+ pointing up in tangent space)
    let normal_sample = textureSample(normal_texture, normal_sampler, tex_coords).rgb;
    let tangent_normal = normal_sample * 2.0 - 1.0;
    // Flip Y to convert from glTF OpenGL convention to rendering convention
    let N = normalize(vec3<f32>(tangent_normal.x * material.normal_scale, -tangent_normal.y * material.normal_scale, tangent_normal.z));

    // Calculate F0 for PBR
    var F0 = vec3<f32>(0.04);
    F0 = mix(F0, albedo, metallic);

    var Lo = vec3<f32>(0.0);
    
    // TBN
    let T = normalize(in.tangent);
    let B = normalize(in.bitangent);
    let N_geom = normalize(in.normal);
    let TBN = transpose(mat3x3<f32>(T, B, N_geom));

    // Directional lights
    for (var i: i32 = 0; i < direct_light_buffer.len; i++) {
        let light = direct_light_buffer.lights[i];
        
        // light direction in tangent space
        let L = normalize(TBN * (-light.direction.xyz));
        let H = normalize(V + L);

        let NdotL = max(dot(N, L), 0.0);

        let radiance = light.color.rgb * light.intensity;

        // Calculate shadow factor
        let shadow = calculate_directional_shadow(light, in.world_pos, N_geom);

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

        // Add to outgoing radiance (apply shadow)
        Lo += (kD * albedo / PI + specular) * radiance * NdotL * shadow;
    }

    // Point lights
    for (var i: i32 = 0; i < point_light_buffer.len; i++) {
        let light = point_light_buffer.lights[i];
        
        // tangent space light pos
        let tangent_light_pos = TBN * light.pos.xyz;
        let L = normalize(tangent_light_pos - in.tangent_frag_pos);
        let H = normalize(V + L);

        let light_distance = length(light.pos.xyz - in.world_pos);
        let attenuation = 1.0 / (light_distance * light_distance);

        let radiance = light.color.rgb * attenuation * light.intensity;

        // Calculate shadow factor
        let shadow = calculate_point_shadow(light, in.world_pos);

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

        // Add to outgoing radiance (apply shadow)
        Lo += (kD * albedo / PI + specular) * radiance * NdotL * shadow;
    }

    // Ambient lighting
    let ao = textureSample(ambient_occlusion_texture, ambient_occlusion_sampler, tex_coords).r;
    let ambient = vec3<f32>(scene.ambient) * albedo * (ao * material.ambient_occlusion_strength);

    // Emissive contribution
    let emissive = textureSample(emissive_texture, emissive_sampler, tex_coords).rgb * material.emissive_factor.rgb;

    // Combine lighting
    var out_color = emissive + ambient + Lo;

    // Tone mapping (Reinhard)
    out_color = out_color / (out_color + vec3<f32>(1.0));

    // Gamma correction
    out_color = pow(out_color, vec3<f32>(1.0 / 2.2));

    // Output world-space normals (after normal mapping)
    // Transform tangent-space normal to world-space using existing TBN
    let TBN_world = mat3x3<f32>(T, B, N_geom);
    let world_normal = normalize(TBN_world * N);

    // Encode normals from [-1, 1] to [0, 1] range for storage
    let encoded_normal = world_normal * 0.5 + 0.5;

    return FragmentOutput(
        vec4<f32>(out_color.rgb, alpha),
        vec4<f32>(encoded_normal, 1.0)
    );
}
