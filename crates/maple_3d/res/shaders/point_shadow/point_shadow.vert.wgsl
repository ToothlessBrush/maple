// Point shadow vertex shader
// Renders geometry from point light's perspective for cube map depth

struct LightData {
    view_projection: mat4x4<f32>,
    light_pos: vec4<f32>,
    far_plane: f32,
    _padding: vec3<f32>,
}

struct MeshData {
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> light: LightData;
@group(1) @binding(0) var<uniform> mesh: MeshData;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tex_uv: vec2<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
}

@vertex
fn main(input: VertexInput) -> VertexOutput {
    // Transform position to world space
    let world_pos = (mesh.model * vec4<f32>(input.position, 1.0)).xyz;

    // Transform position to light's clip space
    let clip_position = light.view_projection * mesh.model * vec4<f32>(input.position, 1.0);

    return VertexOutput(clip_position, world_pos);
}
