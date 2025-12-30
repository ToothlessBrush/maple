@group(0) @binding(0) var environment_map: texture_cube<f32>;
@group(0) @binding(1) var environment_sampler: sampler;

struct Uniforms {
    face_index: u32,
}

@group(0) @binding(2) var<uniform> uniforms: Uniforms;

struct FragmentInput {
    @location(0) local_pos: vec3<f32>,
}

const PI: f32 = 3.14159265359;

fn get_cube_direction(uv: vec2<f32>, face: u32) -> vec3<f32> {
    // Convert UV (-1 to 1) to 3D direction based on cube face
    let u = uv.x;
    let v = uv.y;

    switch face {
        case 0u: { return normalize(vec3<f32>(1.0, -v, -u)); }   // +X
        case 1u: { return normalize(vec3<f32>(-1.0, -v, u)); }   // -X
        case 2u: { return normalize(vec3<f32>(u, 1.0, v)); }     // +Y
        case 3u: { return normalize(vec3<f32>(u, -1.0, -v)); }   // -Y
        case 4u: { return normalize(vec3<f32>(u, -v, 1.0)); }    // +Z
        default: { return normalize(vec3<f32>(-u, -v, -1.0)); }  // -Z
    }
}

@fragment
fn main(input: FragmentInput) -> @location(0) vec4<f32> {
    let normal: vec3<f32> = get_cube_direction(input.local_pos.xy, uniforms.face_index);

    var irradiance: vec3<f32> = vec3(0.0);

    // Build tangent space basis - handle pole case
    var up: vec3<f32> = vec3(0.0, 1.0, 0.0);
    if (abs(normal.y) > 0.999) {
        up = vec3(1.0, 0.0, 0.0); // Use X as up when normal is parallel to Y
    }
    let right: vec3<f32> = normalize(cross(up, normal));
    let up_corrected: vec3<f32> = normalize(cross(normal, right));

    // Reduce sample_delta for higher quality (less noise)
    let sample_delta: f32 = 0.01;  // Increased from 0.025 for smoother results
    var nr_samples: f32 = 0.0;

    for (var phi: f32 = 0.0; phi < 2.0 * PI; phi += sample_delta) {
        for (var theta: f32 = 0.0; theta < 0.5 * PI; theta += sample_delta) {
            // Spherical to cartesian (in tangent space)
            let tangent_sample = vec3(
                sin(theta) * cos(phi),
                sin(theta) * sin(phi),
                cos(theta)
            );

            // Tangent space to world
            let sample_vec = tangent_sample.x * right + tangent_sample.y * up_corrected + tangent_sample.z * normal;

            // Flip Y for cubemap sampling (WebGPU coordinate system)
            let flipped_sample = sample_vec * vec3<f32>(1.0, -1.0, 1.0);

            // Sample and clamp to prevent fireflies from extreme HDR values
            let sample_color = textureSample(environment_map, environment_sampler, flipped_sample).rgb;
            let clamped_color = min(sample_color, vec3<f32>(10.0));  // Clamp bright spots

            irradiance += clamped_color * cos(theta) * sin(theta);
            nr_samples += 1.0;
        }
    }

    irradiance = PI * irradiance * (1.0 / nr_samples);

    return vec4(irradiance, 1.0);
}
