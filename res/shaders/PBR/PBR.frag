#version 430 core

const int MAX_LIGHTS = 2;
const float PI = 3.14159265359;

out vec4 fragColor;

in vec3 crntPos;
in vec3 v_normal;
in vec4 v_Color;
in vec2 v_TexCoord;
in vec3 v_tangent;
in vec3 v_bitangent;

uniform vec3 camPos;

uniform bool u_LightingEnabled;

uniform vec3 u_BackgroundColor;

struct Scene {
    float biasFactor;
    float biasOffset;
    float ambient;
};

uniform Scene scene;

struct MaterialProperties {
    vec4 baseColorFactor;
    bool useTexture;
    sampler2D baseColorTexture; // rgba

    float metallicFactor;
    float roughnessFactor;
    bool useMetallicRoughnessTexture;
    sampler2D metallicRoughnessTexture; // metallic on blue channel and roughness on green

    float normalScale;
    bool useNormalTexture;
    sampler2D normalTexture; // the normal defines a vec3 relavent to tangent space and scaled by normal scale

    float ambientOcclusionStrength;
    bool useOcclusionTexture;
    sampler2D occlusionTexture; // defines areas that are occluded from light

    vec3 emissiveFactor;
    bool useEmissiveTexture;
    sampler2D emissiveTexture; // object may glow

    // other properties
    bool useAlphaCutoff;
    float alphaCutoff; // value that the alpha channel is cutoff in mask mode
    bool doubleSided;
};

uniform MaterialProperties material;

struct PointLight {
    vec4 color;
    vec4 pos;
    float intensity;
    int shadowIndex; // index into samplerCubeArray
    float far_plane;
    int _padding;
};

layout(std430, binding = 1) readonly buffer pointLights {
    int pointLightsLength;
    PointLight pointLight[];
};

struct DirectLight {
    vec4 color;
    vec4 direction;
    float intensity;
    int shadowIndex;
    int cascadeLevel;
    float farPlane;
    float cascadeSplit[4];
    mat4 lightSpaceMatrices[4];
};

layout(std430, binding = 0) readonly buffer directLights {
    int directLightsLength;
    DirectLight directLight[];
};

//uniform DirectLight directLights[10];
//uniform int directLightLength;

// uniform PointLight pointLights[10];
// uniform int pointLightLength;

uniform samplerCubeArray shadowCubeMaps;
uniform sampler2DArray shadowMaps;

/* <=======================================>
 *
 * End of Uniform Definitions
 *
 * <=======================================>
 */

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
    // base color
    vec4 baseColor = material.useTexture
        ? texture(material.baseColorTexture, v_TexCoord) : vec4(1.0);

    vec3 albedo = pow(baseColor.rgb, vec3(2.2)) * material.baseColorFactor.rgb;
    float alpha = baseColor.a * material.baseColorFactor.a;

    if (material.useAlphaCutoff && alpha < material.alphaCutoff) {
        discard;
    }

    // Metallic roughness
    float metallic = material.metallicFactor;
    float roughness = material.roughnessFactor;
    if (material.useMetallicRoughnessTexture) {
        vec4 mrSample = texture(material.metallicRoughnessTexture, v_TexCoord);
        metallic = mrSample.b * material.metallicFactor;
        roughness = mrSample.g * material.roughnessFactor;
    }
    // Normal mapping
    vec3 N = normalize(v_normal); // The surface normal in world space (or object space)

    if (material.useNormalTexture) {
        mat3 TBN = mat3(
                normalize(v_tangent),
                normalize(v_bitangent),
                normalize(v_normal)
            );

        // Sample the normal from the normal map (assumed to be in world/object space)
        vec3 normalMap = texture(material.normalTexture, v_TexCoord).xyz * 2.0 - 1.0;
        N = normalize(TBN * normalMap);
    }

    // view direction
    vec3 V = normalize(camPos - crntPos);

    // Ambient occlusion
    float ao = material.useOcclusionTexture
        ? texture(material.occlusionTexture, v_TexCoord).r * material.ambientOcclusionStrength : material.ambientOcclusionStrength;

    vec3 emissive = material.useEmissiveTexture
        ? texture(material.emissiveTexture, v_TexCoord).r * material.emissiveFactor : material.emissiveFactor;

    //calculate F0
    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedo, metallic);

    vec3 Lo = vec3(0.0);
    // directional lights (light direction is the same for every fragment and shadows are cascaded)
    for (int i = 0; i < directLightsLength; i++) {
        // Light vector
        vec3 L = normalize(directLight[i].direction.xyz);
        // half way vector
        vec3 H = normalize(V + L);

        float NdotL = max(dot(N, L), 0.0);
        // vec3 diffuse = directLight[i].color.rgb * NdotL;

        vec3 radiance = directLight[i].color.rgb * directLight[i].intensity;

        vec3 F = FresnelSchlick(max(dot(H, V), 0.0), F0);

        // cook-torrance BRDF
        float NDF = DistributionShlickGGX(N, H, roughness);
        float G = GeometrySmith(N, V, L, roughness);

        vec3 numerator = NDF * G * F;
        float denomintator = 4.0 * max(dot(N, V), 0.0) * NdotL + 0.0001;
        vec3 specular = numerator / denomintator;

        // apply metallic and roughness factors
        vec3 ks = F;
        vec3 kD = vec3(1.0) - ks;
        kD *= 1.0 - metallic;

        // shadow
        // get cascade level
        float distance = length(camPos - crntPos);
        int cascadeLevel = directLight[i].cascadeLevel - 1; // Default to the last cascade
        for (int y = 0; y < directLight[i].cascadeLevel; y++) {
            float radius = (directLight[i].farPlane / 2) * directLight[i].cascadeSplit[y];
            if (distance < radius) {
                cascadeLevel = y;
                break;
            }
        }

        // calculate fragment position on shadow map
        vec4 fragPosLight = directLight[i].lightSpaceMatrices[cascadeLevel] * vec4(crntPos, 1.0f); // first convert the fragment to light space (so we can match it with the shadow map)
        vec3 projCoords = fragPosLight.xyz / fragPosLight.w; // extract the 3d cords
        projCoords = (projCoords + 1.0f) / 2.0f; // convert from -1-1 to 0-1 coordnates
        float currentDepth = projCoords.z; // depth of the fragment
        vec2 shadowMapUV = projCoords.xy; // position of the fragment

        // shadowMapUV = clamp(shadowMapUV, 0.4, 0.6);
        int cascadeIndex = max(directLight[i].shadowIndex + cascadeLevel, 0);

        // float bias = scene.biasFactor * (1.0 - NdotL) + scene.biasOffset; // 0.000006 * (1.0 - NdotL) + 0.000002;
        float bias = mix(scene.biasOffset, scene.biasOffset + scene.biasFactor, 1.0 - NdotL) * (directLight[i].cascadeSplit[cascadeLevel] / directLight[i].cascadeSplit[0]);

        int range = 2;
        float shadow = 0.0f;
        float pixelSize = 1.0 / textureSize(shadowMaps, 0).x; // Adjust according to your shadow map size

        for (int y = -range; y <= range; y++) {
            for (int x = -range; x <= range; x++) {
                // Calculate the offset based on the pixel size
                vec2 offsetUV = shadowMapUV + vec2(x, y) * pixelSize;

                // Ensure the offset UV is within bounds to avoid indexing out of bounds
                if (offsetUV.x >= 0.0 && offsetUV.x <= 1.0 && offsetUV.y >= 0.0 && offsetUV.y <= 1.0) {
                    // Sample the shadow map at the offset UV
                    float closestDepth = texture(shadowMaps, vec3(offsetUV, cascadeIndex)).r;
                    // Add the comparison to shadow
                    shadow += step(closestDepth + bias, currentDepth);
                }
            }
        }

        // // Normalize the shadow value by the total number of samples
        shadow /= float((range * 2 + 1) * (range * 2 + 1));

        // combine lighting
        vec3 lighting = (kD * albedo.rgb / PI + specular) * radiance * NdotL * (1.0 - shadow);

        Lo += lighting;
    }

    for (int i = 0; i < pointLightsLength; i++) {
        vec3 L = normalize(pointLight[i].pos.xyz - crntPos);
        vec3 lightToFrag = normalize(crntPos - pointLight[i].pos.xyz);
        vec3 H = normalize(V + L);

        float light_distance = length(pointLight[i].pos.xyz - crntPos);
        float attenuation = 1.0 / (light_distance * light_distance); // inverse square law

        vec3 radiance = pointLight[i].color.rgb * attenuation * pointLight[i].intensity;

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

        // shadow maps should be square if they arent then we have other issues
        float pixelSize = 1.0f / textureSize(shadowMaps, 0).x; // Adjust according to your shadow map size

        int sampleRadius = 2;
        float shadow = 0.0f;

        float bias = mix(scene.biasOffset, scene.biasOffset + scene.biasFactor, 1.0 - NdotL);
        for (int z = -sampleRadius; z <= sampleRadius; z++) {
            for (int y = -sampleRadius; y <= sampleRadius; y++) {
                for (int x = -sampleRadius; x <= sampleRadius; x++) {
                    vec3 sampleDir = normalize(lightToFrag + vec3(x, y, z) * pixelSize);
                    float closestDepth = texture(shadowCubeMaps, vec4(sampleDir, pointLight[i].shadowIndex)).r;

                    closestDepth *= pointLight[i].far_plane;
                    if (light_distance > closestDepth + bias) {
                        shadow += 1.0f;
                    }
                }
            }
        }
        shadow /= pow((sampleRadius * 2 + 1), 3);

        vec3 lighting = (kD * albedo.rgb / PI + specular) * radiance * NdotL * (1.0 - shadow);
        Lo += lighting;
    }

    vec3 ambient = vec3(scene.ambient) * albedo.rgb * ao;
    vec3 outColor = emissive + ambient + Lo;

    outColor = outColor / (outColor + vec3(1.0));
    outColor = pow(outColor, vec3(1.0 / 2.2));

    fragColor = vec4(outColor.rgb, alpha);
}
