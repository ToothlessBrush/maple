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

@fragment
fn main(input: FragmentInput) {
    // For point lights, we could store the distance in the depth buffer
    // or just let the hardware depth testing handle it
    // The depth value is automatically written

    // Note: If we wanted to store distance instead of depth, we would:
    // let distance = length(input.world_pos - light.light_pos.xyz);
    // let normalized_distance = distance / light.far_plane;
    // But for now, standard depth testing is sufficient
}
