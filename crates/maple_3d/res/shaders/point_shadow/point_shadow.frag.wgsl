// Point shadow fragment shader
// Calculate and store distance from light for point light shadows

struct LightData {
    view_projection: mat4x4<f32>,
    light_pos: vec4<f32>,
    far_plane: f32,
    _padding: vec3<f32>,
}

@group(0) @binding(0) var<uniform> light: LightData;

struct FragmentInput {
    @builtin(position) frag_coord: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
}

struct FragmentOutput {
    @builtin(frag_depth) depth: f32,
}

@fragment
fn main(input: FragmentInput) -> FragmentOutput {
    // Store the linear distance from light to fragment
    // This needs to match what we compare in the main shader
    let distance = length(input.world_pos - light.light_pos.xyz);
    let normalized_distance = distance / light.far_plane;

    var output: FragmentOutput;
    output.depth = normalized_distance;
    return output;
}
