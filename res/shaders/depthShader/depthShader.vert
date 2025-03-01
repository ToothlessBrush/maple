#version 330 core
layout(location = 0) in vec3 position;

//uniform mat4 u_lightSpaceMatrix;
uniform mat4 u_Model;

out vec2 v_texCoords;

struct Light {
    vec3 direction;
    mat4 matrices[4];
};

uniform int cascadeNumber;

uniform Light light;

void main() {
    gl_Position = light.matrices[cascadeNumber] * u_Model * vec4(position, 1.0);
    v_texCoords = position.xy * 0.5 + 0.5;
}