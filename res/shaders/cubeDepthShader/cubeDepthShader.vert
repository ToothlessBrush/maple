#version 430 core
layout (location = 0) in vec3 aPos;

uniform mat4 u_Model;

out vec2 v_texCoords;

void main()
{
    gl_Position = u_Model * vec4(aPos, 1.0);
    v_texCoords = aPos.xy * 0.5 + 0.5;
}