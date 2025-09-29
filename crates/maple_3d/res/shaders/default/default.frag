#version 450 core

layout(location = 0) out vec4 fragColor;

layout(location = 0) in vec3 crntPos;
layout(location = 2) in vec2 v_TexCoord;

// Descriptor Set 0: Scene Data
layout(set = 0, binding = 0) uniform SceneData {
    vec4 backgroundColor;
    float ambient;
} scene;

// Descriptor Set 0: Camera Data
layout(set = 0, binding = 1) uniform CameraData {
    vec4 camPos;
    mat4 projection;
    mat4 view;
    mat4 VP; // view * projection
} camera;

// Descriptor Set 1: Material Data
layout(set = 1, binding = 0) uniform MaterialData {
    vec4 base_color_factor;
    float metallic_factor;
    float roughness_factor;
    float normal_scale;
    float ambient_occlusion_strength;
    vec4 emissive_factor;
    float alpha_cutoff;
} material;

// Descriptor Set 2: Mesh Data
layout(set = 2, binding = 0) uniform MeshData {
    mat4 model;
} mesh;

void main() {
    // Base color from material
    vec4 baseColor = material.base_color_factor;

    vec3 albedo = pow(baseColor.rgb, vec3(2.2)); // Convert to linear space
    float alpha = baseColor.a;

    // Alpha cutoff test (hardcoded for now since no alpha mode in buffer)
    if (alpha < material.alpha_cutoff) {
        discard;
    }

    // Simple ambient lighting
    vec3 ambient = vec3(scene.ambient) * albedo.rgb * material.ambient_occlusion_strength;

    // Add emissive contribution (using .xyz to get vec3 from vec4)
    vec3 emissive = material.emissive_factor.xyz;

    // Combine ambient and emissive
    vec3 outColor = ambient + emissive;

    // Tone mapping (Reinhard)
    outColor = outColor / (outColor + vec3(1.0));

    // Gamma correction
    outColor = pow(outColor, vec3(1.0 / 2.2));

    fragColor = vec4(outColor.rgb, alpha);
}
