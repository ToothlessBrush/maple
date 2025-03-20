#version 430 core

const int MAX_LIGHTS = 2;

out vec4 fragColor;

in vec3 crntPos;
in vec3 v_normal;
in vec4 v_Color;
in vec2 v_TexCoord;

// uniform sampler2D u_albedoMap;
// uniform sampler2D u_specularMap;
// uniform sampler2D shadowMap;

//uniform samplerCube shadowCubeMap;

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

vec4 get_texture_value(sampler2D unit, vec2 texCoord, vec4 default_value) {
    return textureSize(unit, 0).x > 1 ? texture(unit, texCoord) : default_value; // return if the texture size isnt 0
}

vec4 get_texture_value(samplerCube unit, vec3 texCoord, vec4 default_value) {
    return textureSize(unit, 0).x > 1 ? texture(unit, texCoord) : default_value;
}

vec4 get_texture_value(sampler2DArray unit, vec2 texCoord, int index, vec4 default_value) {
    return textureSize(unit, 0).x > 1 ? texture(unit, vec3(texCoord, index)) : default_value;
}

vec4 get_texture_value(samplerCubeArray unit, vec3 texCoord, int index, vec4 default_value) {
    return textureSize(unit, 0).x > 1 ? texture(unit, vec4(texCoord, index)) : default_value;
}

vec4 calculate_point_light(PointLight light, vec4 baseColor, float specColor) {
    vec3 lightVec = light.pos - crntPos;
    float dist = length(lightVec);
    float a = 0.1f;
    float b = 0.02f;
    float inten = light.intensity / (a * dist * dist + b * dist + 1.0f);

    // if intensity is really low then disregard light data
    if (inten < 0.001f) {
        return vec4(0.0f); // Return black color (0.0 intensity)
    }

    // ambient light
    // float ambient = 0.05f;

    // diffuse light
    vec3 normal = normalize(v_normal);
    vec3 lightDirection = normalize(lightVec);
    float diffuse = max(dot(normal, lightDirection), 0.0f);

    // specular light blinn-phong
    float specular = 0.0f;
    if (diffuse != 0.0f) // Only calculate specular if there is diffuse light
    {
        vec3 viewDirection = normalize(camPos - crntPos);
        vec3 reflectionDirection = reflect(-lightDirection, normal);
        vec3 halfwayVec = normalize(lightDirection + viewDirection);
        float specAmount = pow(max(dot(normal, halfwayVec), 0.0f), 16);
        specular = specAmount * material.metallicFactor;
    }

    float shadow = 0.0;
    vec3 fragToLight = crntPos - light.pos;
    float currentDepth = length(fragToLight);
    float bias = 0.000006f * (1.0f - dot(normal, lightDirection)) + 0.000002;
    // float bias = u_bias;

    int sampleRadius = 2;
    float pixelSize = 1.0f / 1024.0f;
    for (int z = -sampleRadius; z <= sampleRadius; z++) {
        for (int y = -sampleRadius; y <= sampleRadius; y++) {
            for (int x = -sampleRadius; x <= sampleRadius; x++) {
                vec3 sampleDir = normalize(fragToLight + vec3(x, y, z) * pixelSize);
                float closestDepth = texture(shadowCubeMaps, vec4(sampleDir, light.shadowIndex)).r;

                closestDepth *= light.far_plane;
                if (currentDepth > closestDepth + bias) {
                    shadow += 1.0f;
                }
            }
        }
    }
    shadow /= pow((sampleRadius * 2 + 1), 3);

    vec4 texColor = baseColor * material.baseColorFactor;

    if (material.useAlphaCutoff && texColor.a < material.alphaCutoff) {
        discard; // Discard fragments below alpha cutoff
    }

    float specMap = specColor;
    vec4 finalColor = (texColor * (diffuse * (1.0f - (shadow / 2)) * inten) + specMap * specular * inten) * light.color;

    return vec4(finalColor.rgb, texColor.a);
}

vec4 calculate_direct_light(DirectLight light, vec4 baseColor, float specColor) {
    vec3 lightVec = normalize(light.direction);
    float inten = light.intensity;

    // diffuse
    vec3 normal = normalize(v_normal);
    vec3 lightDir = lightVec;
    float diffuse = max(dot(normal, lightDir), 0.0f);

    // specular light
    float specular = 0.0f;
    if (diffuse != 0.0) {
        vec3 viewDir = (camPos - crntPos);
        vec3 reflectionDir = reflect(-lightDir, normal);
        vec3 halfwayVec = normalize(lightDir + viewDir);
        float specAmount = pow(max(dot(normal, halfwayVec), 0.0f), 16);
        specular = specAmount * material.metallicFactor;
    }

    // shadow
    // get cascade level
    float distance = length(camPos - crntPos);
    int cascadeLevel = light.cascadeLevel - 1; // Default to the last cascade
    for (int i = 0; i < light.cascadeLevel; i++) {
        float radius = (light.farPlane / 2) * light.cascadeSplit[i];
        if (distance < radius) {
            cascadeLevel = i;
            break;
        }
    }

    // calculate fragment position on shadow map
    vec4 fragPosLight = light.lightSpaceMatrices[cascadeLevel] * vec4(crntPos, 1.0f); // first convert the fragment to light space (so we can match it with the shadow map)
    vec3 projCoords = fragPosLight.xyz / fragPosLight.w; // extract the 3d cords
    projCoords = (projCoords + 1.0f) / 2.0f; // convert from -1-1 to 0-1 coordnates
    float currentDepth = projCoords.z; // depth of the fragment
    vec2 shadowMapUV = projCoords.xy; // position of the fragment

    // shadowMapUV = clamp(shadowMapUV, 0.4, 0.6);
    int cascadeIndex = max(light.shadowIndex + cascadeLevel, 0);

    float bias = 0.000006 * (1.0 - dot(normal, lightDir)) + 0.000002;

    int range = 2;
    float shadow = 0.0f;
    float pixelSize = 1.0 / textureSize(shadowMaps, 0).x;

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

    // Normalize the shadow value by the total number of samples
    shadow /= float((range * 2 + 1) * (range * 2 + 1));

    //closestDepth = texture(shadowMaps, vec3(shadowMapUV, cascadeIndex)).r;

    // float closestDepth = 1.0;

    // get if the fragment is in shadow

    //final
    vec4 texColor = baseColor * material.baseColorFactor;

    if (material.useAlphaCutoff && texColor.a < material.alphaCutoff) {
        discard;
    }

    float specMap = specColor;
    vec4 finalColor = (texColor * (diffuse * (1.0f - shadow) * inten) + specMap * specular * inten) * light.color;

    return vec4(finalColor.rgb, texColor.a);
}

// vec4 spotLight() {
//     float outerCone = 0.90f;
//     float innerCone = 0.95f;

//     // ambient light
//     float ambient = 0.20f;

//     // diffuse light
//     vec3 normal = normalize(v_normal);
//     vec3 lightDirection = normalize(lightPos - crntPos);
//     float diffuse = max(dot(normal, lightDirection), 0.0f);

//     //specular light blinn-phong
//     float specular = 0.0f;
//     if (diffuse != 0.0f) // Only calculate if there is diffuse light
//     {
//         vec3 viewDirection = normalize(camPos - crntPos);
//         vec3 reflectionDirection = reflect(-lightDirection, normal);
//         vec3 halfwayVec = normalize(lightDirection + viewDirection);
//         float specAmount = pow(max(dot(normal, halfwayVec), 0.0f), 16);
//         specular = specAmount * u_SpecularStrength;
//     }

//     float angle = dot(vec3(0.0f, -1.0f, 0.0f), -lightDirection);
//     float inten = clamp((angle - outerCone) / (innerCone - outerCone), 0.0f, 1.0f);

//     vec4 texColor = useTexture ? texture(u_albedoMap, v_TexCoord) * baseColorFactor : baseColorFactor;
//     //vec4 texColor = texture(diffi)
//     //vec4 texColor = texture(diffuse0, v_TexCoord);
//     float specMap = texture(u_specularMap, v_TexCoord).r;
//     vec4 finalColor = (texColor * (diffuse * inten + ambient) + specMap * specular * inten) * lightColor;

//     return vec4(finalColor.rgb, texColor.a); // Preserve alpha
// }

float near = 0.1f;
float far = 100.0f;

float linearizeDepth(float depth) {
    return (2.0f * near * far) / (far + near - (depth * 2.0 - 1.0) * (far - near));
}

float logisticDepth(float depth, float steepness, float offset) {
    float zVal = linearizeDepth(depth);
    return (1 / (1 + exp(-steepness * (zVal - offset))));
}

void main() {
    vec4 baseColorTexture = material.useTexture ? texture(material.baseColorTexture, v_TexCoord) : vec4(1.0);
    float specularTexture = material.useMetallicRoughnessTexture ? texture(material.metallicRoughnessTexture, v_TexCoord).b : 1.0; // use mult identity

    if (!u_LightingEnabled) {
        fragColor = baseColorTexture * material.baseColorFactor;
        return;
    }

    float ambientFactor = 0.2f;

    vec4 ambientLight = baseColorTexture * material.baseColorFactor * ambientFactor;

    //vec4 directLightColor = directLight();  // Separate color and alpha
    vec4 LightColor = vec4(ambientLight); //default (ambient) light

    // directional lights
    for (int i = 0; i < directLightLength; i++) {
        LightColor += calculate_direct_light(directLights[i], baseColorTexture, specularTexture);
    }

    //point lights
    for (int i = 0; i < pointLightLength; i++) {
        LightColor += calculate_point_light(pointLights[i], baseColorTexture, specularTexture);
    }

    // vec4 color = texture(shadowMaps, vec3(v_TexCoord, directLights[0].shadowIndex));

    // delete for bloom since values can exceed 1
    clamp(LightColor, 0.0, 1.0);

    float depth = logisticDepth(gl_FragCoord.z, 0.2f, 100.0f);

    // vec4 pointLightColor = pointLight(pointLights[0]);
    vec3 depthColor = (1.0f - depth) + depth * u_BackgroundColor;
    vec3 finalColor = LightColor.rgb * depthColor; //(1.0f - depth) + depth * u_BackgroundColor;

    fragColor = vec4(finalColor, (baseColorTexture * material.baseColorFactor).a); // fragColor is the fragment in the framebuffer
    // fragColor = vec4(color.rgb, texColor.a);
}
