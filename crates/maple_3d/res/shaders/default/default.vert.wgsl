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

struct MeshData {
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> scene: SceneData;
@group(0) @binding(1) var<uniform> camera: CameraData;
@group(2) @binding(0) var<uniform> mesh: MeshData;

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
    @location(1) normal: vec3<f32>,
    @location(2) tex_coord: vec2<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
    @location(5) tangent_view_pos: vec3<f32>,
    @location(6) tangent_frag_pos: vec3<f32>,
}

@vertex
fn main(input: VertexInput) -> VertexOutput {
    // Transform position to world space
    let world_pos = (mesh.model * vec4<f32>(input.position, 1.0)).xyz;
    
    // Transform position to clip space
    let clip_position = camera.VP * mesh.model * vec4<f32>(input.position, 1.0);
    
    // Transform normals
    let normal = normalize((mesh.normal_matrix * vec4<f32>(input.normal, 0.0)).xyz);
    let tangent = normalize((mesh.normal_matrix * vec4<f32>(input.tangent, 0.0)).xyz);
    let bitangent = normalize((mesh.normal_matrix * vec4<f32>(input.bitangent, 0.0)).xyz);
    
    // TBN
    let TBN = transpose(mat3x3<f32>(tangent, bitangent, normal));

    let tangent_view_pos = TBN * camera.cam_pos.xyz;
    let tangent_frag_pos = TBN * world_pos;

    return VertexOutput(
        clip_position,
        world_pos,
        normal,
        input.tex_uv,
        tangent,
        bitangent,
        tangent_view_pos,
        tangent_frag_pos,
    );
}
