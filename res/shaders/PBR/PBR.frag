#version 330 core
// fragment shader output for PBR model

out vec4 FragColor;

in vec3 fragPos;
in vec3 normal;
in vec2 TexCoords;

uniform vec3 camPos;
uniform vec3 lightPos;

uniform vec4 color_factor; // color factor for the mesh material

uniform sampler2D albedoMap; //base color in RGBA channels
uniform sampler2D normalMap; // normal map in RGB channels
uniform sampler2D metallicRoughnessMap; // metallic in R channel, roughness in G channel
uniform sampler2D emissiveMap; // emissive map in RGB channels

vec3 albedoMesh = texture(albedoMap, TexCoords).rgb;
vec3 normalMesh = texture(normalMap, TexCoords).rgb;
float metallicMesh = texture(metallicRoughnessMap, TexCoords).r;
float roughnessMesh = texture(metallicRoughnessMap, TexCoords).g;
vec3 emissiveMesh = texture(emissiveMap, TexCoords).rgb;

// main vectors
vec3 N = normalize(normal);
vec3 V = normalize(camPos - fragPos);
// for point/spot lights
vec3 L = normalize(lightPos - fragPos);
vec3 H = normalize(V + L);

const float PI = 3.14159265359;

// GGX/trowbridge-reitz normal distribution function
float D(float alpha, vec3 N, vec3 H) {
    float numerator = pow(alpha, 2.0);

    float nDotH = max(dot(N, H), 0.0);
    float denominator = PI * pow(pow(nDotH, 2.0) * (pow(alpha, 2.0) - 1.0) + 1.0, 2.0);
    denominator = max(denominator, 0.000001); // prevent division by zero
    return numerator / denominator;
}

// schlick-beckmann geometric shadowing function
float G1(float alpha, vec3 N, vec3 X) {
    float numerator = max(dot(N, X), 0.0);

    float k = alpha / 2.0;
    float denominator = numerator * (1.0 - k) + k;
    denominator = max(denominator, 0.000001); // prevent division by zero
    return numerator / denominator;
}

// smith model
float G(float alpha, vec3 N, vec3 V, vec3 L) {
    return G1(alpha, N, V) * G1(alpha, N, L);
}

//fresnel-schlick function
vec3 F(vec3 F0, vec3 V, vec3 H) {
    return F0 + (vec3(1.0) - F0) * pow(1.0 - max(dot(V, H), 0.0), 5.0);
}

// rendering eq for 1 light source
vec3 PBR() {
    //implement the PBR model here
}

void main() {
    vec3 color = PBR();
    FragColor = vec4(color, 1.0); // preserve alpha
}








