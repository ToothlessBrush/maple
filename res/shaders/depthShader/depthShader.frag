#version 330 core

in vec2 v_texCoords;
uniform sampler2D u_albedoMap;
uniform vec4 u_baseColor;
uniform bool u_hasTexture;

void main() {
    float alpha = 0.0;
    
    if (u_hasTexture) {
        alpha = texture(u_albedoMap, v_texCoords).a;
    } else {
        alpha = u_baseColor.a;
    }
    
    if (alpha < 0.5) { // default discard threshold
        discard;
    }
}