struct CameraData {
    position: vec4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
    vp: mat4x4<f32>,
    far_plane: f32,
}

@group(0) @binding(0) var<uniform> camera: CameraData;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local_pos: vec3<f32>,
}

@vertex
fn main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    // Create a cube using vertex index
    var positions = array<vec3<f32>, 36>(
        // Front face
        vec3<f32>(-1.0, -1.0, 1.0),
        vec3<f32>(1.0, -1.0, 1.0),
        vec3<f32>(1.0, 1.0, 1.0),
        vec3<f32>(1.0, 1.0, 1.0),
        vec3<f32>(-1.0, 1.0, 1.0),
        vec3<f32>(-1.0, -1.0, 1.0),
        // Back face
        vec3<f32>(-1.0, -1.0, -1.0),
        vec3<f32>(-1.0, 1.0, -1.0),
        vec3<f32>(1.0, 1.0, -1.0),
        vec3<f32>(1.0, 1.0, -1.0),
        vec3<f32>(1.0, -1.0, -1.0),
        vec3<f32>(-1.0, -1.0, -1.0),
        // Top face
        vec3<f32>(-1.0, 1.0, -1.0),
        vec3<f32>(-1.0, 1.0, 1.0),
        vec3<f32>(1.0, 1.0, 1.0),
        vec3<f32>(1.0, 1.0, 1.0),
        vec3<f32>(1.0, 1.0, -1.0),
        vec3<f32>(-1.0, 1.0, -1.0),
        // Bottom face
        vec3<f32>(-1.0, -1.0, -1.0),
        vec3<f32>(1.0, -1.0, -1.0),
        vec3<f32>(1.0, -1.0, 1.0),
        vec3<f32>(1.0, -1.0, 1.0),
        vec3<f32>(-1.0, -1.0, 1.0),
        vec3<f32>(-1.0, -1.0, -1.0),
        // Right face
        vec3<f32>(1.0, -1.0, -1.0),
        vec3<f32>(1.0, 1.0, -1.0),
        vec3<f32>(1.0, 1.0, 1.0),
        vec3<f32>(1.0, 1.0, 1.0),
        vec3<f32>(1.0, -1.0, 1.0),
        vec3<f32>(1.0, -1.0, -1.0),
        // Left face
        vec3<f32>(-1.0, -1.0, -1.0),
        vec3<f32>(-1.0, -1.0, 1.0),
        vec3<f32>(-1.0, 1.0, 1.0),
        vec3<f32>(-1.0, 1.0, 1.0),
        vec3<f32>(-1.0, 1.0, -1.0),
        vec3<f32>(-1.0, -1.0, -1.0),
    );

    var output: VertexOutput;
    let pos = positions[vertex_index];

    // Build mat3x3 rotation, then expand to mat4x4
    var rot_mat3 = mat3x3<f32>(
        camera.view[0].xyz,
        camera.view[1].xyz,
        camera.view[2].xyz
    );

    var rot_view = mat4x4<f32>(
        vec4<f32>(rot_mat3[0], 0.0),
        vec4<f32>(rot_mat3[1], 0.0),
        vec4<f32>(rot_mat3[2], 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );

    let clip_pos = camera.projection * rot_view * vec4<f32>(pos, 1.0);

    // Set depth to 1.0 (background) by using xyww trick
    output.position = clip_pos.xyww;

    // Pass the position as the sampling direction
    output.local_pos = pos;

    return output;
}
