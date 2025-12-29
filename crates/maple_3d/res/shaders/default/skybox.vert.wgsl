struct CameraData {
    projection: mat4x4<f32>,
    view: mat4x4<f32>,
    position: vec3<f32>,
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
        vec3<f32>(-1.0, -1.0,  1.0),
        vec3<f32>( 1.0, -1.0,  1.0),
        vec3<f32>( 1.0,  1.0,  1.0),
        vec3<f32>( 1.0,  1.0,  1.0),
        vec3<f32>(-1.0,  1.0,  1.0),
        vec3<f32>(-1.0, -1.0,  1.0),
        // Back face
        vec3<f32>(-1.0, -1.0, -1.0),
        vec3<f32>(-1.0,  1.0, -1.0),
        vec3<f32>( 1.0,  1.0, -1.0),
        vec3<f32>( 1.0,  1.0, -1.0),
        vec3<f32>( 1.0, -1.0, -1.0),
        vec3<f32>(-1.0, -1.0, -1.0),
        // Top face
        vec3<f32>(-1.0,  1.0, -1.0),
        vec3<f32>(-1.0,  1.0,  1.0),
        vec3<f32>( 1.0,  1.0,  1.0),
        vec3<f32>( 1.0,  1.0,  1.0),
        vec3<f32>( 1.0,  1.0, -1.0),
        vec3<f32>(-1.0,  1.0, -1.0),
        // Bottom face
        vec3<f32>(-1.0, -1.0, -1.0),
        vec3<f32>( 1.0, -1.0, -1.0),
        vec3<f32>( 1.0, -1.0,  1.0),
        vec3<f32>( 1.0, -1.0,  1.0),
        vec3<f32>(-1.0, -1.0,  1.0),
        vec3<f32>(-1.0, -1.0, -1.0),
        // Right face
        vec3<f32>( 1.0, -1.0, -1.0),
        vec3<f32>( 1.0,  1.0, -1.0),
        vec3<f32>( 1.0,  1.0,  1.0),
        vec3<f32>( 1.0,  1.0,  1.0),
        vec3<f32>( 1.0, -1.0,  1.0),
        vec3<f32>( 1.0, -1.0, -1.0),
        // Left face
        vec3<f32>(-1.0, -1.0, -1.0),
        vec3<f32>(-1.0, -1.0,  1.0),
        vec3<f32>(-1.0,  1.0,  1.0),
        vec3<f32>(-1.0,  1.0,  1.0),
        vec3<f32>(-1.0,  1.0, -1.0),
        vec3<f32>(-1.0, -1.0, -1.0),
    );

    var output: VertexOutput;
    let pos = positions[vertex_index];

    // Remove translation from view matrix by taking only the rotation part
    // View matrix is column-major: [col0, col1, col2, position]
    // Keep first 3 columns (rotation), replace 4th column (translation) with identity
    var rot_view = mat4x4<f32>(
        vec4<f32>(camera.view[0].xyz, 0.0),
        vec4<f32>(camera.view[1].xyz, 0.0),
        vec4<f32>(camera.view[2].xyz, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );

    let clip_pos = camera.projection * rot_view * vec4<f32>(pos, 1.0);

    // Set depth to 1.0 (background) by using xyww trick
    output.position = vec4<f32>(clip_pos.xy, clip_pos.w, clip_pos.w);

    // Pass the position as the sampling direction
    output.local_pos = pos;

    return output;
}
