#version 430 core
in vec4 FragPos;
in vec2 g_texCoords;


uniform vec3 lightPos;
uniform float farPlane;

uniform sampler2D u_albedoMap;
uniform vec4 u_baseColor;
uniform bool u_hasTexture;

void main() {
    float alpha = 0.0;
    if (u_hasTexture) {
        alpha = texture(u_albedoMap, g_texCoords).a;
    } else {
        alpha = u_baseColor.a;
    }

    if (alpha < 0.5) {
        discard;
    }

    gl_FragDepth = length(FragPos.xyz - lightPos) / farPlane;
}