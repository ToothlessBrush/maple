#version 330 core
layout(location = 0) in vec3 position;

uniform mat4 u_lightSpaceMatrix;
uniform mat4 u_Model;

void main() {
    gl_Position = u_lightSpaceMatrix * u_Model * vec4(position, 1.0);
}