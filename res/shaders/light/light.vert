#version 330 core

layout(location = 0) in vec4 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec4 color;
layout(location = 3) in vec2 texCoord;

out vec4 v_Color;

uniform mat4 u_VP;
uniform mat4 u_Model;

// ran for every vertex
void main() {
    gl_Position = u_VP * u_Model * position;
    v_Color = color;
}