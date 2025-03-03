#version 430 core
layout (triangles) in;
layout (triangle_strip, max_vertices = 18) out;

uniform mat4 shadowMatrices[6];

uniform int index;

in vec2 v_texCoords[];  // Receives from vertex shader
out vec2 g_texCoords;   // Pass to fragment shader

out vec4 FragPos;

void main()
{   
    int offset = index * 6;
    for(int face = 0; face < 6; ++face)
    {
        gl_Layer = offset + face;
        for(int i = 0; i < 3; ++i)
        {
            FragPos = gl_in[i].gl_Position;
            gl_Position = shadowMatrices[face] * FragPos;
            g_texCoords = v_texCoords[i];
            EmitVertex();
        }
        EndPrimitive();
    }
}