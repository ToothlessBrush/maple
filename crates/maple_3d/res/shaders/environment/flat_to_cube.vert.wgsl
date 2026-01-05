struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_pos: vec3<f32>,
}

@vertex
fn main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    // Fullscreen triangle
    let x = f32((vertex_index & 1u) << 2u) - 1.0;
    let y = f32((vertex_index & 2u) << 1u) - 1.0;

    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    
    // Pass through clip coordinates as local position
    // The fragment shader will handle face direction
    out.local_pos = vec3<f32>(x, y, 1.0);

    return out;
}
