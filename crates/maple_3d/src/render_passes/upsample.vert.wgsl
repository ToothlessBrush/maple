struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
}

@vertex
fn main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;

    // Fullscreen triangle trick
    // Generates a triangle that covers the entire screen using vertex indices
    let x = f32((vertex_index & 1u) << 1u);
    let y = f32((vertex_index & 2u));

    out.position = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    out.tex_coord = vec2<f32>(x, 1.0 - y);

    return out;
}
