@group(0) @binding(0) var color_texture: texture_2d<f32>;
@group(0) @binding(1) var color_sampler: sampler;

@fragment
fn main(@location(0) tex_coord: vec2<f32>) -> @location(0) vec4<f32> {
    // Simple blit - sample the resolved color texture and output to surface
    // This is the foundation for future post-processing effects like:
    // - Tone mapping (HDR -> LDR)
    // - Bloom (glow for bright areas)
    // - Color grading (LUT-based)
    // - FXAA (fast approximate anti-aliasing)
    return textureSample(color_texture, color_sampler, tex_coord);
}
