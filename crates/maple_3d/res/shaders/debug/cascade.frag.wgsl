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
}

@fragment
fn main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Get the first directional light (assuming you have at least one)
    if direct_light_buffer.len == 0 {
        return vec4<f32>(1.0, 0.0, 1.0, 1.0); // Magenta = no lights
    }

    let light = direct_light_buffer.lights[0];

    if light.shadow_index < 0 {
        return vec4<f32>(0.5, 0.5, 0.5, 1.0); // Gray = no shadow
    }

    // Select cascade based on depth (same as your actual code)
    let view_pos = camera.view * vec4<f32>(in.world_pos, 1.0);
    let depth = abs(view_pos.z);

    var cascade_index = -1;
    for (var i = 0; i < light.cascade_level; i++) {
        if depth < light.cascade_split[i] {
            cascade_index = i;
            break;
        }
    }
    
    // black if no cascade
    if cascade_index == -1 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    // Transform to light space
    let light_space_pos = light.light_space_matrices[cascade_index] * vec4<f32>(in.world_pos, 1.0);
    var proj_coords = light_space_pos.xyz / light_space_pos.w;

    // Transform to [0, 1] range
    proj_coords = proj_coords * 0.5 + 0.5;

    // Determine cascade color
    var cascade_color: vec3<f32>;
    if cascade_index == 0 {
        cascade_color = vec3<f32>(1.0, 0.0, 0.0); // Red = cascade 0 (nearest)
    } else if cascade_index == 1 {
        cascade_color = vec3<f32>(0.0, 1.0, 0.0); // Green = cascade 1
    } else if cascade_index == 2 {
        cascade_color = vec3<f32>(0.0, 0.0, 1.0); // Blue = cascade 2
    } else if cascade_index == 3 {
        cascade_color = vec3<f32>(1.0, 1.0, 0.0); // Yellow = cascade 3 (farthest)
    } else {
        cascade_color = vec3<f32>(1.0, 0.0, 1.0); // Magenta = invalid cascade
    }

    // Check if outside shadow map bounds
    if proj_coords.x < 0.0 || proj_coords.x > 1.0 || proj_coords.y < 0.0 || proj_coords.y > 1.0 || proj_coords.z > 1.0 || proj_coords.z < 0.0 {
        // Washed out version of cascade color (desaturate and brighten)
        let washed_out = mix(cascade_color, vec3<f32>(1.0, 1.0, 1.0), 0.7);
        return vec4<f32>(washed_out, 1.0);
    }

    // Full saturated color = within bounds
    return vec4<f32>(cascade_color, 1.0);
}
