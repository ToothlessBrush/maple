#version 430 core

const int MAX_LIGHTS = 2;
const float PI = 3.14159265359;

out vec4 fragColor;

in vec3 crntPos;
in vec3 v_normal;
in vec4 v_Color;
in vec2 v_TexCoord;

uniform vec3 camPos;

uniform bool u_LightingEnabled;

uniform vec3 u_BackgroundColor;

uniform float ambientLight;

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
    vec3 pos;
    float intensity;
    int shadowIndex; // index into samplerCubeArray
    float far_plane;
};

struct DirectLight {
    vec4 color;
    vec3 direction;
    float intensity;
    int shadowIndex;
    int cascadeLevel;
    float cascadeSplit[4];
    mat4 lightSpaceMatrices[4];
    float farPlane;
};

uniform DirectLight directLights[10];
uniform int directLightLength;

uniform PointLight pointLights[10];
uniform int pointLightLength;

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
    vec4 albedo = material.useTexture
        ? texture(material.baseColorTexture, v_TexCoord) * material.baseColorFactor : material.baseColorFactor;

    if (material.useAlphaCutoff && albedo.a < material.alphaCutoff) {
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
    vec3 N = normalize(v_normal);  // The surface normal in world space (or object space)
    
    if (material.useNormalTexture) {
        // Sample the normal from the normal map (assumed to be in world/object space)
        vec3 normalFromMap = texture(material.normalTexture, v_TexCoord).xyz * 2.0 - 1.0;
        
        // The final normal is a blend of the vertex normal and the normal from the map
        // Multiply the sampled normal by the vertex normal to get the final normal
        N = normalize(normalFromMap + N); // Combine the sampled normal with the vertex normal
    }
  
    // view direction
    vec3 V = normalize(camPos - crntPos);

    // Ambient occlusion
    float ao = material.useOcclusionTexture
        ? texture(material.occlusionTexture, v_TexCoord).r * material.ambientOcclusionStrength : material.ambientOcclusionStrength;


    //calculate F0
    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedo.rgb, metallic);
    
    vec3 Lo = vec3(0.0);
    // directional lights (light direction is the same for every fragment and shadows are cascaded)
    for (int i = 0; i < directLightLength; i++) {
        // Light vector
        vec3 L = normalize(directLights[i].direction);
        // half way vector
        vec3 H = normalize(V + L);

        float NdotL = max(dot(N, L), 0.0);
        vec3 diffuse = directLights[i].color.rgb * NdotL;

        vec3 F = FresnelSchlick(max(dot(H, V), 0.0), F0);

        // cook-torrance BRDF
        float NDF = DistributionShlickGGX(N, H, roughness);
        float G = GeometrySmith(N, V, L, roughness);
        vec3 specular = (NDF * G * F) / max(4.0 * max(dot(N, V), 0.0) * NdotL, 0.001);

        // apply metallic and roughness factors
        vec3 ks = F;
        vec3 kD = vec3(1.0) - ks;
        kD *= 1.0 - metallic;

        // shadow calculation
        float shadow = 1.0; // todo!

        // combine lighting
        vec3 lighting = (kD * diffuse + specular) * directLights[i].intensity * shadow;

        // apply color and ambient occlusion
        lighting *= albedo.rgb * ao;

        Lo += lighting;
    }

    vec3 ambient = vec3(0.03) * albedo.rgb * ao;
    vec3 outColor = ambient + Lo;
      
    fragColor = vec4(outColor.rgb, albedo.a);
}
