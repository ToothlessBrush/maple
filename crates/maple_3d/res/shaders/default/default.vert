#version 450 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 tex_uv;
layout(location = 3) in vec3 tangent;
layout(location = 4) in vec3 bitangent;

layout(location = 0) out vec3 crntPos;
layout(location = 1) out vec3 v_normal;
layout(location = 2) out vec2 v_TexCoord;
layout(location = 3) out vec3 v_tangent;
layout(location = 4) out vec3 v_bitangent;

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

// Descriptor Set 2: Mesh Data
layout(set = 2, binding = 0) uniform MeshData {
    mat4 model;
} mesh;

void main() {
    mat4 normalMatrix = transpose(inverse(mesh.model));

    // Transform position to world space
    crntPos = vec3(mesh.model * vec4(position, 1.0));
    v_TexCoord = tex_uv;

    // Transform position to clip space
    gl_Position = camera.VP * mesh.model * vec4(position, 1.0);

    v_normal = normalize((normalMatrix * vec4(normal, 0.0)).xyz);
    v_tangent = normalize((normalMatrix * vec4(tangent, 0.0)).xyz);
    v_bitangent = normalize((normalMatrix * vec4(bitangent, 0.0)).xyz);
}
