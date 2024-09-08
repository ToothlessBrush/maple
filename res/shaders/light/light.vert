#version 330 core

layout (location = 0) in vec4 position;
layout (location = 1) in vec4 color;

out vec4 v_Color;

uniform mat4 u_VP;
uniform mat4 u_Model;

// ran for every vertex
void main() {
    gl_Position = u_VP * u_Model * position;
    v_Color = color;
}