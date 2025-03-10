#version 330 core

in vec2 g_texCoords;
uniform sampler2D u_albedoMap;
uniform vec4 u_baseColor;
uniform bool u_hasTexture;

struct Light {
    vec3 direction;
    int cascadeCount;
    mat4 matrices[4];
};

uniform Light lights;

void main() {
    float alpha = 0.0;
    
    if (u_hasTexture) {
        alpha = texture(u_albedoMap, g_texCoords).a;
    } else {
        alpha = u_baseColor.a;
    }
    
    if (alpha < 0.5) { // default discard threshold
        discard;
    }
}