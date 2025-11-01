#version 450 core

const float PI = 3.14159265359;

layout(location = 0) out vec4 fragColor;

layout(location = 0) in vec3 crntPos;
layout(location = 1) in vec3 v_normal;
layout(location = 2) in vec2 v_TexCoord;
layout(location = 3) in vec3 v_tangent;
layout(location = 4) in vec3 v_bitangent;

// Descriptor Set 0: Scene Data
layout(set = 0, binding = 0) uniform SceneData {
    vec4 backgroundColor;
    float ambient;
} scene;

// Descriptor Set 0: Camera Data
layout(set = 0, binding = 1) uniform CameraData {
    vec4 camPos;
    mat4 projection;
    mat4 view;
    mat4 VP; // view * projection
} camera;

// Descriptor Set 1: Material Data
layout(set = 1, binding = 0) uniform MaterialData {
    vec4 base_color_factor;
    float metallic_factor;
    float roughness_factor;
    float normal_scale;
    float ambient_occlusion_strength;
    vec4 emissive_factor;
    float alpha_cutoff;
} material;

// Descriptor Set 2: Mesh Data
layout(set = 2, binding = 0) uniform MeshData {
    mat4 model;
} mesh;

struct DirectLight {
    vec4 color;
    vec4 direction;
    float intensity;
    int shadowIndex;
    int cascadeLevel;
    float farPlane;
    vec4 cascadeSplit;
    mat4 lightSpaceMatrices[4];
};

struct PointLight {
    vec4 color;
    vec4 pos;
    float intensity;
    int shadowIndex;
    float far_plane;
    int _padding;
};

// Descriptor Set 3: Light Data
layout(std430, set = 3, binding = 0) restrict readonly buffer DirectLightBuffer {
    int len;
    DirectLight directLights[];
} directLightBuffer;

layout(std430, set = 3, binding = 1) restrict readonly buffer PointLightBuffer {
    int len;
    PointLight pointLights[];
} pointLightBuffer;

// PBR Functions
float DistributionShlickGGX(vec3 N, vec3 H, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;
    float NdotH = max(dot(N, H), 0.0);
    float NdotH2 = NdotH * NdotH;

    float num = a2;
    float denom = (NdotH2 * (a2 - 1.0) + 1.0);
    denom = PI * denom * denom;

    return num / denom;
}

float GeometrySchlickGGX(float NdotV, float roughness) {
    float r = (roughness + 1.0);
    float k = (r * r) / 8.0;

    float num = NdotV;
    float denom = NdotV * (1.0 - k) + k;

    return num / denom;
}

float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness) {
    float NdotV = max(dot(N, V), 0.0);
    float NdotL = max(dot(N, L), 0.0);
    float ggx2 = GeometrySchlickGGX(NdotV, roughness);
    float ggx1 = GeometrySchlickGGX(NdotL, roughness);

    return ggx1 * ggx2;
}

vec3 FresnelSchlick(float cosTheta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

void main() {
    // Base color from material
    vec4 baseColor = material.base_color_factor;
    vec3 albedo = pow(baseColor.rgb, vec3(2.2)); // Convert to linear space
    float alpha = baseColor.a;

    // Alpha cutoff test
    if (alpha < material.alpha_cutoff) {
        discard;
    }

    // Material properties
    float metallic = material.metallic_factor;
    float roughness = material.roughness_factor;

    // Normal (no normal mapping for now)
    vec3 N = normalize(v_normal);

    // View direction
    vec3 V = normalize(camera.camPos.xyz - crntPos);

    // Calculate F0 for PBR
    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedo, metallic);
    vec3 Lo = vec3(0.0);

    // Directional lights
    for (int i = 0; i < directLightBuffer.len; i++) {
        vec3 L = normalize(directLightBuffer.directLights[i].direction.xyz);
        vec3 H = normalize(V + L);

        float NdotL = max(dot(N, L), 0.0);

        vec3 radiance = directLightBuffer.directLights[i].color.rgb * directLightBuffer.directLights[i].intensity;

        // Cook-Torrance BRDF
        float NDF = DistributionShlickGGX(N, H, roughness);
        float G = GeometrySmith(N, V, L, roughness);
        vec3 F = FresnelSchlick(max(dot(H, V), 0.0), F0);

        vec3 numerator = NDF * G * F;
        float denominator = 4.0 * max(dot(N, V), 0.0) * NdotL + 0.0001;
        vec3 specular = numerator / denominator;

        // Energy conservation
        vec3 kS = F;
        vec3 kD = vec3(1.0) - kS;
        kD *= 1.0 - metallic;

        // Add to outgoing radiance
        Lo += (kD * albedo / PI + specular) * radiance * NdotL;
    }

    // Point lights
    for (int i = 0; i < pointLightBuffer.len; i++) {
        vec3 L = normalize(pointLightBuffer.pointLights[i].pos.xyz - crntPos);
        vec3 H = normalize(V + L);

        float light_distance = length(pointLightBuffer.pointLights[i].pos.xyz - crntPos);
        float attenuation = 1.0 / (light_distance * light_distance);

        vec3 radiance = pointLightBuffer.pointLights[i].color.rgb * attenuation * pointLightBuffer.pointLights[i].intensity;

        // Cook-Torrance BRDF
        float NDF = DistributionShlickGGX(N, H, roughness);
        float G = GeometrySmith(N, V, L, roughness);
        vec3 F = FresnelSchlick(max(dot(H, V), 0.0), F0);

        vec3 kS = F;
        vec3 kD = vec3(1.0) - kS;
        kD *= 1.0 - metallic;

        vec3 numerator = NDF * G * F;
        float denominator = 4.0 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.0001;
        vec3 specular = numerator / denominator;

        float NdotL = max(dot(N, L), 0.0);

        // Add to outgoing radiance
        Lo += (kD * albedo / PI + specular) * radiance * NdotL;
    }

    // Ambient lighting
    vec3 ambient = vec3(scene.ambient) * albedo * material.ambient_occlusion_strength;

    // Emissive contribution
    vec3 emissive = material.emissive_factor.xyz;

    // Combine lighting
    vec3 outColor = emissive + ambient + Lo;

    // Tone mapping (Reinhard)
    outColor = outColor / (outColor + vec3(1.0));

    // Gamma correction
    outColor = pow(outColor, vec3(1.0 / 2.2));

    fragColor = vec4(outColor.rgb, alpha);
}
