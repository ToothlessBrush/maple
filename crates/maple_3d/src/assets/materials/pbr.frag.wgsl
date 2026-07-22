const PI: f32 = 3.14159265359;

const ALPHA_MODE_OPAQUE: u32 = 0u;
const ALPHA_MODE_MASK: u32 = 1u;
const ALPHA_MODE_BLEND: u32 = 2u;

struct SceneData {
    background_color: vec4<f32>,
    ambient: f32,
    ibl_strength: f32,
}

struct CameraData {
    cam_pos: vec4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    VP: mat4x4<f32>,
    far_plane: f32}

struct MaterialData {
    base_color_factor: vec4<f32>,
    metallic_factor: f32,
    roughness_factor: f32,
    normal_scale: f32,
    ambient_occlusion_strength: f32,
    emissive_factor: vec4<f32>,
    alpha_cutoff: f32,
    parallax_scale: f32,
    alpha_mode: u32,
    unlit: u32,
    texture_scale: vec2<f32>,
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
    cascade_texel_size: array<f32, 4>,
    size: f32,
    normal_bias: f32,
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
@group(0) @binding(2) var irradiance_map: texture_cube<f32>;
@group(0) @binding(3) var irradiance_sampler: sampler;
@group(0) @binding(4) var prefilter_map: texture_cube<f32>;
@group(0) @binding(5) var prefilter_sampler: sampler;
@group(0) @binding(6) var brdf_lut: texture_2d<f32>;
@group(0) @binding(7) var brdf_lut_sampler: sampler;

@group(1) @binding(0) var<storage, read> mesh: array<MeshData>;

@group(2) @binding(0) var<storage, read> direct_light_buffer: DirectLightBuffer;
@group(2) @binding(1) var<storage, read> point_light_buffer: PointLightBuffer;
@group(2) @binding(2) var directional_shadow_maps: texture_depth_2d_array;
@group(2) @binding(3) var point_shadow_maps: texture_depth_cube_array;
@group(2) @binding(4) var shadow_sampler: sampler_comparison;
@group(2) @binding(5) var shadow_sampler_linear: sampler;

@group(3) @binding(0) var<uniform> material: MaterialData;
@group(3) @binding(1) var base_color_texture: texture_2d<f32>;
@group(3) @binding(2) var base_color_sampler: sampler;
@group(3) @binding(3) var metallic_roughness_texture: texture_2d<f32>;
@group(3) @binding(4) var metallic_roughness_sampler: sampler;
@group(3) @binding(5) var ambient_occlusion_texture: texture_2d<f32>;
@group(3) @binding(6) var ambient_occlusion_sampler: sampler;
@group(3) @binding(7) var emissive_texture: texture_2d<f32>;
@group(3) @binding(8) var emissive_sampler: sampler;
@group(3) @binding(9) var normal_texture: texture_2d<f32>;
@group(3) @binding(10) var normal_sampler: sampler;

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

fn fresnel_schlick_roughness(cosTheta: f32, F0: vec3<f32>, roughness: f32) -> vec3<f32> {
    return F0 + (max(vec3(1.0 - roughness), F0) - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}

fn f0_from_ior(ior: f32) -> f32 {
    let f = (ior - 1.0) / (ior + 1.0);
    return f * f;
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

// shadow sampling techniques were taken from bevy
fn sample_shadow_map_castano_thirteen(light_local: vec2<f32>, depth: f32, array_index: i32) -> f32 {
    let shadow_map_size = vec2<f32>(textureDimensions(directional_shadow_maps));
    let inv_shadow_map_size = 1.0 / shadow_map_size;
    let uv = light_local * shadow_map_size;
    var base_uv = floor(uv + 0.5);
    let s = (uv.x + 0.5 - base_uv.x);
    let t = (uv.y + 0.5 - base_uv.y);
    base_uv -= 0.5;
    base_uv *= inv_shadow_map_size;

    let uw0 = (4.0 - 3.0 * s);
    let uw1 = 7.0;
    let uw2 = (1.0 + 3.0 * s);
    let u0 = (3.0 - 2.0 * s) / uw0 - 2.0;
    let u1 = (3.0 + s) / uw1;
    let u2 = s / uw2 + 2.0;

    let vw0 = (4.0 - 3.0 * t);
    let vw1 = 7.0;
    let vw2 = (1.0 + 3.0 * t);
    let v0 = (3.0 - 2.0 * t) / vw0 - 2.0;
    let v1 = (3.0 + t) / vw1;
    let v2 = t / vw2 + 2.0;

    var sum = 0.0;
    sum += uw0 * vw0 * textureSampleCompareLevel(directional_shadow_maps, shadow_sampler, base_uv + (vec2(u0, v0) * inv_shadow_map_size), array_index, depth);
    sum += uw1 * vw0 * textureSampleCompareLevel(directional_shadow_maps, shadow_sampler, base_uv + (vec2(u1, v0) * inv_shadow_map_size), array_index, depth);
    sum += uw2 * vw0 * textureSampleCompareLevel(directional_shadow_maps, shadow_sampler, base_uv + (vec2(u2, v0) * inv_shadow_map_size), array_index, depth);
    sum += uw0 * vw1 * textureSampleCompareLevel(directional_shadow_maps, shadow_sampler, base_uv + (vec2(u0, v1) * inv_shadow_map_size), array_index, depth);
    sum += uw1 * vw1 * textureSampleCompareLevel(directional_shadow_maps, shadow_sampler, base_uv + (vec2(u1, v1) * inv_shadow_map_size), array_index, depth);
    sum += uw2 * vw1 * textureSampleCompareLevel(directional_shadow_maps, shadow_sampler, base_uv + (vec2(u2, v1) * inv_shadow_map_size), array_index, depth);
    sum += uw0 * vw2 * textureSampleCompareLevel(directional_shadow_maps, shadow_sampler, base_uv + (vec2(u0, v2) * inv_shadow_map_size), array_index, depth);
    sum += uw1 * vw2 * textureSampleCompareLevel(directional_shadow_maps, shadow_sampler, base_uv + (vec2(u1, v2) * inv_shadow_map_size), array_index, depth);
    sum += uw2 * vw2 * textureSampleCompareLevel(directional_shadow_maps, shadow_sampler, base_uv + (vec2(u2, v2) * inv_shadow_map_size), array_index, depth);

    return sum * (1.0 / 144.0);
}

const SPIRAL_OFFSET_0_ = vec2<f32>(-0.7071, 0.7071);
const SPIRAL_OFFSET_1_ = vec2<f32>(-0.0000, -0.8750);
const SPIRAL_OFFSET_2_ = vec2<f32>(0.5303, 0.5303);
const SPIRAL_OFFSET_3_ = vec2<f32>(-0.6250, -0.0000);
const SPIRAL_OFFSET_4_ = vec2<f32>(0.3536, -0.3536);
const SPIRAL_OFFSET_5_ = vec2<f32>(-0.0000, 0.3750);
const SPIRAL_OFFSET_6_ = vec2<f32>(-0.1768, -0.1768);
const SPIRAL_OFFSET_7_ = vec2<f32>(0.1250, 0.0000);

fn interleaved_gradient_noise(pixel_coordinates: vec2<f32>, frame: u32) -> f32 {
    let xy = pixel_coordinates + 5.588238 * f32(frame % 64u);
    return fract(52.9829189 * fract(0.06711056 * xy.x + 0.00583715 * xy.y));
}

fn random_rotation_matrix(scale: vec2<f32>, temporal: bool) -> mat2x2<f32> {
    let random_angle = 2.0 * PI * interleaved_gradient_noise(
        scale, select(1u, 1u, temporal)
    );
    let m = vec2(sin(random_angle), cos(random_angle));
    return mat2x2(
        m.y, -m.x,
        m.x, m.y
    );
}

fn map(min1: f32, max1: f32, min2: f32, max2: f32, value: f32) -> f32 {
    return min2 + (value - min1) * (max2 - min2) / (max1 - min1);
}

// Calculates the distance between spiral samples for the given texel size and
// penumbra size. This is used for the Jimenez '14 (i.e. temporal) variant of
// shadow sampling.
fn calculate_uv_offset_scale_jimenez_fourteen(texel_size: f32, blur_size: f32) -> vec2<f32> {
    let shadow_map_size = vec2<f32>(textureDimensions(directional_shadow_maps));

    // Empirically chosen fudge factor to make PCF look better across different CSM cascades
    let f = map(0.00390625, 0.022949219, 0.015, 0.035, texel_size);
    return f * blur_size / (texel_size * shadow_map_size);
}

fn sample_shadow_map_jimenez_fourteen(
    light_local: vec2<f32>,
    depth: f32,
    array_index: i32,
    frag_coord_xy: vec2<f32>,
    texel_size: f32,
    blur_size: f32,
    temporal: bool,
) -> f32 {
    let rotation_matrix = random_rotation_matrix(frag_coord_xy, temporal);
    let uv_offset_scale = calculate_uv_offset_scale_jimenez_fourteen(texel_size, blur_size);

    // https://www.iryoku.com/next-generation-post-processing-in-call-of-duty-advanced-warfare (slides 120-135)
    let sample_offset0 = (rotation_matrix * SPIRAL_OFFSET_0_) * uv_offset_scale;
    let sample_offset1 = (rotation_matrix * SPIRAL_OFFSET_1_) * uv_offset_scale;
    let sample_offset2 = (rotation_matrix * SPIRAL_OFFSET_2_) * uv_offset_scale;
    let sample_offset3 = (rotation_matrix * SPIRAL_OFFSET_3_) * uv_offset_scale;
    let sample_offset4 = (rotation_matrix * SPIRAL_OFFSET_4_) * uv_offset_scale;
    let sample_offset5 = (rotation_matrix * SPIRAL_OFFSET_5_) * uv_offset_scale;
    let sample_offset6 = (rotation_matrix * SPIRAL_OFFSET_6_) * uv_offset_scale;
    let sample_offset7 = (rotation_matrix * SPIRAL_OFFSET_7_) * uv_offset_scale;

    var sum = 0.0;
    sum += textureSampleCompareLevel(directional_shadow_maps, shadow_sampler, light_local + sample_offset0, array_index, depth);
    sum += textureSampleCompareLevel(directional_shadow_maps, shadow_sampler, light_local + sample_offset1, array_index, depth);
    sum += textureSampleCompareLevel(directional_shadow_maps, shadow_sampler, light_local + sample_offset2, array_index, depth);
    sum += textureSampleCompareLevel(directional_shadow_maps, shadow_sampler, light_local + sample_offset3, array_index, depth);
    sum += textureSampleCompareLevel(directional_shadow_maps, shadow_sampler, light_local + sample_offset4, array_index, depth);
    sum += textureSampleCompareLevel(directional_shadow_maps, shadow_sampler, light_local + sample_offset5, array_index, depth);
    sum += textureSampleCompareLevel(directional_shadow_maps, shadow_sampler, light_local + sample_offset6, array_index, depth);
    sum += textureSampleCompareLevel(directional_shadow_maps, shadow_sampler, light_local + sample_offset7, array_index, depth);
    return sum / 8.0;
}

fn search_for_blockers_in_shadow_map_hardware(
    light_local: vec2<f32>,
    depth: f32,
    array_index: i32,
) -> vec2<f32> {
    let sampled_depth = textureSampleLevel(
        directional_shadow_maps,
        shadow_sampler_linear,
        light_local,
        array_index,
        0u,
    );

    return select(vec2(0.0), vec2(sampled_depth, 1.0), sampled_depth >= depth);
}

// These are the standard MSAA sample point positions from D3D. They were chosen
// to get a reasonable distribution that's not too regular.
//
// https://learn.microsoft.com/en-us/windows/win32/api/d3d11/ne-d3d11-d3d11_standard_multisample_quality_levels?redirectedfrom=MSDN
const D3D_SAMPLE_POINT_POSITIONS: array<vec2<f32>, 8> = array(
    vec2(0.125, -0.375),
    vec2(-0.125, 0.375),
    vec2(0.625, 0.125),
    vec2(-0.375, -0.625),
    vec2(-0.625, 0.625),
    vec2(-0.875, -0.125),
    vec2(0.375, 0.875),
    vec2(0.875, -0.875),
);

fn search_for_blockers_in_shadow_map(
    light_local: vec2<f32>,
    depth: f32,
    array_index: i32,
    texel_size: f32,
    search_size: f32,
) -> f32 {
    let shadow_map_size = vec2<f32>(textureDimensions(directional_shadow_maps));
    let uv_offset_scale = search_size / (texel_size * shadow_map_size);

    let offset0 = D3D_SAMPLE_POINT_POSITIONS[0] * uv_offset_scale;
    let offset1 = D3D_SAMPLE_POINT_POSITIONS[1] * uv_offset_scale;
    let offset2 = D3D_SAMPLE_POINT_POSITIONS[2] * uv_offset_scale;
    let offset3 = D3D_SAMPLE_POINT_POSITIONS[3] * uv_offset_scale;
    let offset4 = D3D_SAMPLE_POINT_POSITIONS[4] * uv_offset_scale;
    let offset5 = D3D_SAMPLE_POINT_POSITIONS[5] * uv_offset_scale;
    let offset6 = D3D_SAMPLE_POINT_POSITIONS[6] * uv_offset_scale;
    let offset7 = D3D_SAMPLE_POINT_POSITIONS[7] * uv_offset_scale;

    var sum = vec2(0.0);

    sum += search_for_blockers_in_shadow_map_hardware(light_local + offset0, depth, array_index);
    sum += search_for_blockers_in_shadow_map_hardware(light_local + offset1, depth, array_index);
    sum += search_for_blockers_in_shadow_map_hardware(light_local + offset2, depth, array_index);
    sum += search_for_blockers_in_shadow_map_hardware(light_local + offset3, depth, array_index);
    sum += search_for_blockers_in_shadow_map_hardware(light_local + offset4, depth, array_index);
    sum += search_for_blockers_in_shadow_map_hardware(light_local + offset5, depth, array_index);
    sum += search_for_blockers_in_shadow_map_hardware(light_local + offset6, depth, array_index);
    sum += search_for_blockers_in_shadow_map_hardware(light_local + offset7, depth, array_index);

    if sum.y == 0.0 {
        return 0.0;
    }

    return sum.x / sum.y;
}

// sample a cascade texture
fn sample_cascade_shadow(
    light: DirectLight,
    world_pos: vec3<f32>,
    normal: vec3<f32>,
    cascade_index: i32,
    frag_coord: vec2<f32>,
    surface_normal: vec3<f32>,
) -> f32 {
    // Transform to light space
    let light_space_matrix = light.light_space_matrices[cascade_index];
    let light_dir = normalize(-light.direction.xyz);

    // Normal + depth offset bias applied in world space before projection
    let normal_offset = light.normal_bias * light.cascade_texel_size[cascade_index] * surface_normal.xyz;
    let depth_offset = light.bias * light_dir.xyz;
    let offset_position = world_pos.xyz + normal_offset + depth_offset;

    let light_space_pos = light_space_matrix * vec4<f32>(offset_position, 1.0);
    var proj_coords = light_space_pos.xyz / light_space_pos.w;

    // Transform XY to [0, 1] range for texture sampling
    proj_coords.x = proj_coords.x * 0.5 + 0.5;
    proj_coords.y = proj_coords.y * 0.5 + 0.5;
    // Flip Y (shadow maps were upside down)
    proj_coords.y = 1.0 - proj_coords.y;

    if proj_coords.x < 0.0 || proj_coords.x > 1.0 || proj_coords.y < 0.0 || proj_coords.y > 1.0 || proj_coords.z > 1.0 {
        return 1.0;
    }

    let depth = proj_coords.z;
    let shadow_layer = light.shadow_index * 4 + cascade_index;
    let shadow_map_dim = textureDimensions(directional_shadow_maps);
    let texel_size = 1.0 / vec2<f32>(shadow_map_dim);

    let z_blocker = search_for_blockers_in_shadow_map(
        proj_coords.xy,
        depth,
        shadow_layer,
        texel_size.x,
        light.size,
    );

    let blur_size = max((z_blocker - depth) * 0.5 / depth, 0.5);

    return sample_shadow_map_castano_thirteen(
        proj_coords.xy,
        depth,
        shadow_layer,
    );

    // return sample_shadow_map_jimenez_fourteen(
    //     proj_coords.xy,
    //     depth,
    //     shadow_layer,
    //     frag_coord,
    //     texel_size.x,
    //     blur_size,
    //     false,
    // );
}
// Calculate shadow factor for directional lights with cascade blending
fn calculate_directional_shadow(light: DirectLight, world_pos: vec3<f32>, normal: vec3<f32>, frag_coord: vec2<f32>) -> f32 {
    if light.shadow_index < 0 {
        return 1.0; // No shadow
    }

    // Select cascade based on depth
    let view_pos = camera.view * vec4<f32>(world_pos, 1.0);
    let depth = abs(view_pos.z);

    var cascade_index = light.cascade_level - 1;
    var blend_factor = 0.0;

    // Find which cascade we're in and calculate blend factor
    for (var i = 0; i < light.cascade_level; i++) {
        if depth < light.cascade_split[i] {
            cascade_index = i;

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

    let shadow = sample_cascade_shadow(light, world_pos, normal, cascade_index, frag_coord, normal);

    // Blend with next cascade if in transition zone
    if blend_factor > 0.0 && cascade_index < light.cascade_level - 1 {
        let next_shadow = sample_cascade_shadow(light, world_pos, normal, cascade_index + 1, frag_coord, normal);
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

    let current_depth = length(light_to_frag);
    let normalized_depth = current_depth / light.far_plane;

    if normalized_depth > 1.0 {
        return 1.0; // Beyond shadow range
    }

    // shadow maps are upside down
    let sample_dir = light_to_frag * vec3<f32>(1.0, -1.0, 1.0);

    let compare_depth = saturate(normalized_depth - light.bias);

    let shadow = textureSampleCompare(
        point_shadow_maps,
        shadow_sampler,
        sample_dir,
        light.shadow_index,
        compare_depth
    );

    return shadow;
}

// we dont do parallax mapping but Ill keep the function
// fn parallax_mapping(tex_coords: vec2<f32>, view_dir: vec3<f32>) -> vec2<f32> {
//     // Number of depth layers
//     let min_layers = 8.0;
//     let max_layers = 32.0;
//     let num_layers = mix(max_layers, min_layers, max(dot(vec3(0.0, 0.0, 1.0), view_dir), 0.0));
// 
//     // Calculate the size of each layer
//     let layer_depth = 1.0 / num_layers;
//     // Depth of current layer
//     var current_layer_depth = 0.0;
//     // The amount to shift the texture coordinates per layer (from vector P)
//     let P = view_dir.xy * material.parallax_scale; // assuming you have this in your material
//     let delta_tex_coords = P / num_layers;
// 
//     // Get initial values
//     var current_tex_coords = tex_coords;
//     var current_depth_map_value = textureSample(parallax_texture, parallax_sampler, current_tex_coords).r;
// 
//     // Iterate until we find a depth value less than the layer's depth
//     while current_layer_depth < current_depth_map_value {
//         // Shift texture coordinates along direction of P
//         current_tex_coords -= delta_tex_coords;
//         // Get depthmap value at current texture coordinates
//         current_depth_map_value = textureSample(parallax_texture, parallax_sampler, current_tex_coords).r;
//         // Get depth of next layer
//         current_layer_depth += layer_depth;
//     }
// 
//     let prev_tex_coords = current_tex_coords + delta_tex_coords;
// 
//     let after_depth = current_depth_map_value - current_layer_depth;
//     let before_depth = textureSample(parallax_texture, parallax_sampler, prev_tex_coords).r - current_layer_depth + layer_depth;
// 
//     let weight = after_depth / (after_depth - before_depth);
//     let final_tex_coords = prev_tex_coords * weight + current_tex_coords * (1.0 - weight);
// 
//     return final_tex_coords;
// }

struct FragmentOutput {
    @location(0) color: vec4<f32>,
    @location(1) normal: vec4<f32>,
}

@fragment
fn main(in: VertexOutput) -> FragmentOutput {

    // View direction in tangent space
    let V = normalize(in.tangent_view_pos - in.tangent_frag_pos);
    // let tex_coords = parallax_mapping(in.tex_coord, V);

    let tex_coords = in.tex_coord * material.texture_scale;

    // Base color from material
    let base_color = textureSample(base_color_texture, base_color_sampler, tex_coords) * material.base_color_factor;
    let albedo = pow(base_color.rgb, vec3<f32>(2.2)); // Convert to linear space
    var alpha = base_color.a;

    if material.alpha_mode == ALPHA_MODE_MASK && alpha < material.alpha_cutoff {
        discard;
    }

    if material.alpha_mode == ALPHA_MODE_OPAQUE {
        alpha = 1.0;
    }

    if material.unlit == 1u {
        let emissive = textureSample(emissive_texture, emissive_sampler, tex_coords).rgb * material.emissive_factor.rgb;
        let unlit_color = albedo + emissive;

        // Gamma correction
        let final_color = pow(unlit_color, vec3<f32>(1.0 / 2.2));

        let encoded_normal = normalize(in.normal) * 0.5 + 0.5;
        return FragmentOutput(
            vec4<f32>(final_color, alpha),
            vec4<f32>(encoded_normal, 1.0)
        );
    }

    let metallic_roughness = textureSample(
        metallic_roughness_texture,
        metallic_roughness_sampler,
        tex_coords
    );

    let metallic = metallic_roughness.b * material.metallic_factor;
    let roughness = metallic_roughness.g * material.roughness_factor;

    let normal_sample = textureSample(normal_texture, normal_sampler, tex_coords).rgb;
    let tangent_normal = normal_sample * 2.0 - 1.0;

    let N = normalize(vec3<f32>(tangent_normal.x * material.normal_scale, -tangent_normal.y * material.normal_scale, tangent_normal.z));

    let T = normalize(in.tangent);
    let B = normalize(in.bitangent);
    let N_geom = normalize(in.normal);
    let TBN = transpose(mat3x3<f32>(T, B, N_geom));
    let TBN_world = mat3x3<f32>(T, B, N_geom);
    let world_normal = normalize(TBN_world * N);

    // check rate of change of roughness in screenspace (helps with specular aliasing)
    // this is so that if there is a single pixel wide rough section it doesnt stand out
    let normal_invariance = length(fwidth(world_normal));
    let roughnes_aa = normal_invariance * 0.5;
    let adjusted_roughness = saturate(roughness + roughnes_aa);

    let NdotV = max(dot(N, V), 0.0);

    if material.alpha_mode == ALPHA_MODE_BLEND {
        let fresnel_factor = pow(1.0 - NdotV, 5.0);

        let fresnel_alpha = mix(base_color.a, 1.0, fresnel_factor * 0.5);

        alpha = fresnel_alpha;
    }

    // Calculate dielectric value for reflectance minimum is 0.04
    var F0 = vec3<f32>(0.04);
    F0 = mix(F0, albedo, metallic);

    var Lo = vec3<f32>(0.0);

    // TBN

    // Directional lights
    for (var i: i32 = 0; i < direct_light_buffer.len; i++) {
        let light = direct_light_buffer.lights[i];

        // macro surface faces away skip shading (fixes light bleeding in normal maps)
        let L_world = normalize(-light.direction.xyz);
        let geom_NdotL = max(dot(N_geom, L_world), 0.0);
        if geom_NdotL <= 0.0 {
            continue;
        }

        // light direction in tangent space
        let L = normalize(TBN * (-light.direction.xyz));
        let H = normalize(V + L);

        let NdotL = max(dot(N, L), 0.0);

        let horizon_fade = smoothstep(0.0, 1.0, geom_NdotL);

        let radiance = light.color.rgb * light.intensity;
        let shadow = calculate_directional_shadow(light, in.world_pos, N_geom, in.clip_position.xy);

        // Cook-Torrance BRDF
        let NDF = distribution_schlick_ggx(N, H, adjusted_roughness);
        let G = geometry_smith(N, V, L, adjusted_roughness);
        let F = fresnel_schlick(max(dot(H, V), 0.0), F0);

        let numerator = NDF * G * F;
        let denominator = 4.0 * max(dot(N, V), 0.0) * NdotL + 0.0001;
        let specular = numerator / denominator;

        let kS = F;
        let kD = (vec3<f32>(1.0) - kS) * (1.0 - metallic);

        // Add to outgoing radiance (apply shadow)
        Lo += (kD * albedo / PI + specular) * radiance * NdotL * shadow * horizon_fade;
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

        // shadowing
        let shadow = calculate_point_shadow(light, in.world_pos);

        // Cook-Torrance BRDF
        let NDF = distribution_schlick_ggx(N, H, adjusted_roughness);
        let G = geometry_smith(N, V, L, adjusted_roughness);
        let F = fresnel_schlick(max(dot(H, V), 0.0), F0);

        let kS = F;
        let kD = (vec3<f32>(1.0) - kS) * (1.0 - metallic);

        let numerator = NDF * G * F;
        let denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
        let specular = numerator / denominator;

        let NdotL = max(dot(N, L), 0.0);

        // Add to outgoing radiance
        Lo += (kD * albedo / PI + specular) * radiance * NdotL * shadow;
    }

    let ao = textureSample(ambient_occlusion_texture, ambient_occlusion_sampler, tex_coords).r;

    let world_view_dir = normalize(camera.cam_pos.xyz - in.world_pos);
    let NdotV_world = max(dot(world_normal, world_view_dir), 0.0);

    // Calculate ambient term
    var ambient = vec3(0.0);

    // IBL
    if scene.ibl_strength > 0.0 {
        // Calculate reflection vector for specular IBL
        let R = reflect(-world_view_dir, world_normal);

        // Fresnel with roughness for IBL
        let kS_ibl = fresnel_schlick_roughness(NdotV_world, F0, adjusted_roughness);
        let kD_ibl = (vec3<f32>(1.0) - kS_ibl) * (1.0 - metallic);

        // diffuse
        let irradiance = textureSample(irradiance_map, irradiance_sampler, world_normal).rgb;
        let diffuse = irradiance * albedo;

        // specular
        let max_reflection_lod = f32(textureNumLevels(prefilter_map) - 1);
        let prefilteredColor = textureSampleLevel(
            prefilter_map,
            prefilter_sampler,
            R,
            adjusted_roughness * max_reflection_lod
        ).rgb;
        let brdf = textureSample(brdf_lut, brdf_lut_sampler, vec2<f32>(NdotV_world, adjusted_roughness)).rg;
        let specular = prefilteredColor * (F0 * brdf.r + brdf.g);

        // Combine diffuse and specular IBL
        let ao_factor = mix(1.0, ao, material.ambient_occlusion_strength);
        ambient = (kD_ibl * diffuse * ao_factor + specular) * scene.ibl_strength;
    } else {
        // Fallback ambient when no IBL is available
        // Use a simple hemisphere lighting approach
        let kS_ambient = fresnel_schlick_roughness(NdotV_world, F0, adjusted_roughness);
        let kD_ambient = (1.0 - metallic) * (vec3<f32>(1.0) - kS_ambient);

        // Apply ambient to diffuse component only (metals don't have diffuse)
        let ao_factor = mix(1.0, ao, material.ambient_occlusion_strength);
        ambient = kD_ambient * albedo * scene.ambient * ao_factor;
    }

    let emissive = textureSample(emissive_texture, emissive_sampler, tex_coords).rgb * material.emissive_factor.rgb;

    var out_color = emissive + ambient + Lo;

    // Encode world-space normals for proper range
    let encoded_normal = world_normal * 0.5 + 0.5;

    return FragmentOutput(
        vec4<f32>(out_color.rgb, alpha),
        vec4<f32>(encoded_normal, 1.0)
    );
}
