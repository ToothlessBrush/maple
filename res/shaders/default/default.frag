#version 430 core

const int MAX_LIGHTS = 2;

out vec4 fragColor;

in vec3 crntPos;
in vec3 v_normal;
in vec4 v_Color;
in vec2 v_TexCoord;
in vec4 fragPosLight;

uniform sampler2D u_albedoMap;
uniform sampler2D u_specularMap;
uniform sampler2D shadowMap;

//uniform samplerCube shadowCubeMap;


uniform vec4 baseColorFactor;

uniform bool useTexture;

uniform bool useAlphaCutoff;
uniform float alphaCutoff;


//uniform vec4 lightColor;
//uniform vec3 lightPos;
uniform vec3 camPos;
//uniform float u_farShadowPlane;
uniform vec3 u_directLightDirection;

//uniform vec3 u_pointLightPosition;

uniform bool u_LightingEnabled;

uniform float farPlane;

uniform float u_SpecularStrength;
uniform float u_AmbientStrength;

uniform float u_bias;

uniform vec3 u_BackgroundColor;

uniform float ambientLight;

struct PointLight {
    vec4 color;
    vec3 pos;
    float intensity;
    int shadowIndex; // index into samplerCubeArray
};

struct DirectLight {
    vec4 color;
    vec3 pos;
    int shadowIndex;
};


uniform DirectLight directLights[10];
uniform int pointLightLength;

uniform PointLight pointLights[10];

uniform samplerCubeArray shadowCubeMaps;
uniform sampler2DArray shadowMaps;


vec4 shadowLight() {
    return texture(shadowMap, v_TexCoord);
    
}

vec4 pointLight(PointLight light) {

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
        specular = specAmount * u_SpecularStrength;
    }

    float shadow = 0.0;
    vec3 fragToLight = crntPos - light.pos;
    float currentDepth = length(fragToLight);
    float bias = max(0.5f * (1.0f - dot(normal, lightDirection)), 0.0005f);
    // float bias = u_bias;

    int sampleRadius  = 0;
    float pixelSize = 1.0f / 1024.0f;
    for (int z = -sampleRadius; z <= sampleRadius; z++) {
        for (int y = -sampleRadius; y <= sampleRadius; y++) {
            for (int x = -sampleRadius; x <= sampleRadius; x++) {
                vec3 sampleDir = normalize(fragToLight + vec3(x, y, z) * pixelSize);
                float closestDepth = texture(shadowCubeMaps, vec4(sampleDir, light.shadowIndex)).r;

                closestDepth *= farPlane;
                if (currentDepth > closestDepth + bias) {
                    shadow += 1.0f;
                }
            }
        }
    }
    shadow /= pow((sampleRadius * 2 + 1), 3);

    
    vec4 texColor = useTexture ? texture(u_albedoMap, v_TexCoord) : baseColorFactor;

    if (useAlphaCutoff && texColor.a < alphaCutoff) {
        discard; // Discard fragments below alpha cutoff
    }

    float specMap = texture(u_specularMap, v_TexCoord).r;
    vec4 finalColor =  (texColor * (diffuse * (1.0f - shadow) * inten) + specMap * specular * inten) * light.color;

    return vec4(finalColor.rgb, texColor.a); // Preserve alpha
}

vec4 directLight(DirectLight light) {
    // Ambient light

    
    // float ambient = 0.20f;
    
    // Diffuse light
    vec3 normal = normalize(v_normal);
    vec3 lightDirection = normalize(u_directLightDirection); // Directional light
    float diffuse = max(dot(normal, lightDirection), 0.0f);

    // Specular light blinn-phong
    float specular = 0.0f;
    if (diffuse != 0.0f) // Only calculate specular if there is diffuse light
    {
        vec3 viewDirection = normalize(camPos - crntPos);
        vec3 reflectionDirection = reflect(-lightDirection, normal);
        vec3 halfwayVec = normalize(lightDirection + viewDirection);
        float specAmount = pow(max(dot(normal, halfwayVec), 0.0f), 16);
        specular = specAmount * u_SpecularStrength;
    }

    float distance = length(light.pos.xyz - fragPosLight.xyz);

    //calculate shadow factor
    float shadow = 0.0f;
    vec3 lightCoords = fragPosLight.xyz / fragPosLight.w;
    if(lightCoords.z <= 1.0f) {
        lightCoords = (lightCoords + 1.0f) / 2.0f;

        float closestDepth = texture(shadowMap, lightCoords.xy).r;
        float currentDepth = lightCoords.z;

        

        //float bias = max(0.05 * (1.0 - dot(normal, lightDirection)), 0.0001); // Bias to prevent shadow acne
        float bias = u_bias;
        //float bias = max(.005f * distance / u_farShadowPlane, u_bias); // Bias to prevent shadow acne but also prevent peter panning
        //soften shadows
        int sampleRadius = 2;
        vec2 pixelSize = 1.0f / textureSize(shadowMap, 0);
        for (int y = -sampleRadius; y <= sampleRadius; y++) {
            for (int x = -sampleRadius; x <= sampleRadius; x++) {
                float closestDepth = texture(shadowMap, lightCoords.xy + vec2(x, y) * pixelSize).r;
                if (currentDepth > closestDepth + bias) {
                    shadow += 1.0f;
                }
            }
        }
        shadow /= pow(sampleRadius * 2.0f + 1.0f, 2.0f);
        
        // if (currentDepth > closestDepth + bias) {
        //     shadow = 1.0f;
        // }
    }

    vec4 texColor = /* vec4(1.0f, 1.0f, 1.0f, texture(diffuse0, v_TexCoord).a); */ useTexture ? texture(u_albedoMap, v_TexCoord) : baseColorFactor;

    if (useAlphaCutoff && texColor.a < alphaCutoff) {
        discard; // Discard fragments below alpha cutoff
    }

    //vec4 texColor = texture(diffuse0, v_TexCoord);
    float specMap = texture(u_specularMap, v_TexCoord).g;

    // Combine textures with lighting
    vec4 finalColor = (texColor * (diffuse * (1.0f - shadow)  /* + ambient */) + specMap * specular * (1.0f - shadow)) * light.color;

    return vec4(finalColor.rgb, texColor.a); // Preserve alpha
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
    if (!u_LightingEnabled) {
        fragColor = useTexture ? texture(u_albedoMap, v_TexCoord) : baseColorFactor;
        return;
    }

    float ambientFactor = 0.05f;

    vec4 texColor = useTexture ? texture(u_albedoMap, v_TexCoord) : baseColorFactor;

    vec4 ambientLight = texColor * ambientFactor;

    //vec4 directLightColor = directLight();  // Separate color and alpha
    vec4 LightColor = vec4(ambientLight); //default (ambient) light
    for (int i = 0; i < pointLightLength; i++) {
        LightColor += pointLight(pointLights[i]);
    }

    // delete for bloom since values can exceed 1
    clamp(LightColor, 0.0, 1.0);

    float depth = logisticDepth(gl_FragCoord.z, 0.2f, 100.0f);
    
    // vec4 pointLightColor = pointLight(pointLights[0]);
    vec3 depthColor = (1.0f - depth) + depth * u_BackgroundColor;
    vec3 finalColor = LightColor.rgb * depthColor;//(1.0f - depth) + depth * u_BackgroundColor;

    fragColor = vec4(finalColor, texColor.a); // fragColor is the fragment in the framebuffer
}