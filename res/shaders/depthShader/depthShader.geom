#version 430 core
layout (triangles) in;
layout (triangle_strip, max_vertices = 18) out;

// uniform mat4 shadowMatrices[4];

in vec2 v_texCoords[];  // Receives from vertex shader
out vec2 g_texCoords;   // Pass to fragment shader

struct Light {
    vec3 direction;
    int index; 
    int cascadeDepth;
    mat4 matrices[4];
};

uniform Light light;

out vec4 FragPos;

void main()
{   
    int offset = light.index;
    for(int face = 0; face < light.cascadeDepth; ++face)
    {
        gl_Layer = offset + face;
        for(int i = 0; i < 3; ++i)
        {
            FragPos = gl_in[i].gl_Position;
            gl_Position = light.matrices[face] * FragPos;
            g_texCoords = v_texCoords[i];
            EmitVertex();
        }
        EndPrimitive();
    }
}