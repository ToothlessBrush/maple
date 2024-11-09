#version 330 core

out vec4 fragColor;

in vec3 crntPos;
in vec3 v_normal;
in vec4 v_Color;
in vec2 v_TexCoord;
in vec4 fragPosLight;

uniform sampler2D diffuse0;
uniform sampler2D specular0;
uniform sampler2D shadowMap;


uniform vec4 baseColorFactor;

uniform bool useTexture;

uniform bool useAlphaCutoff;
uniform float alphaCutoff;

uniform vec4 lightColor;
uniform vec3 lightPos;
uniform vec3 camPos;

uniform float u_SpecularStrength;
uniform float u_AmbientStrength;

uniform vec3 u_BackgroundColor;

vec4 shadowLight() {
    return texture(shadowMap, v_TexCoord);
    
}

vec4 pointLight() {
    vec3 lightVec = lightPos - crntPos;
    float dist = length(lightVec);
    float a = 0.1f;
    float b = 0.02f;
    float inten = 1.0f / (a * dist * dist + b * dist + 1.0f);

    // ambient light
    float ambient = 0.20f;
    
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

    
    vec4 texColor = useTexture ? texture(diffuse0, v_TexCoord) : baseColorFactor;
    float specMap = texture(specular0, v_TexCoord).r;
    vec4 finalColor =  (texColor * (diffuse * inten + ambient) + specMap * specular * inten) * lightColor;

    return vec4(finalColor.rgb, texColor.a); // Preserve alpha
}

vec4 directLight() {
    // Ambient light

    
    float ambient = 0.20f;
    
    // Diffuse light
    vec3 normal = normalize(v_normal);
    vec3 lightDirection = normalize(vec3(1.0f, 1.0f, 1.0f)); // Directional light
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

    //calculate shadow factor
    float shadow = 0.0f;
    vec3 lightCoords = fragPosLight.xyz / fragPosLight.w;
    if(lightCoords.z <= 1.0f) {
        lightCoords = (lightCoords + 1.0f) / 2.0f;

        float closestDepth = texture(shadowMap, lightCoords.xy).r;
        float currentDepth = lightCoords.z;

        float bias = max(0.025f * (1.0f - dot(normal, lightDirection)), 0.0005f); // Bias to prevent shadow acne
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

    vec4 texColor = useTexture ? texture(diffuse0, v_TexCoord) : baseColorFactor;

    if (useAlphaCutoff && texColor.a < alphaCutoff) {
        discard; // Discard fragments below alpha cutoff
    }

    //vec4 texColor = texture(diffuse0, v_TexCoord);
    float specMap = texture(specular0, v_TexCoord).g;

    // Combine textures with lighting
    vec4 finalColor = (texColor * (diffuse * (1.0f - shadow) + ambient) + specMap * specular * (1.0f - shadow)) * lightColor;

    return vec4(finalColor.rgb, texColor.a); // Preserve alpha
}

vec4 spotLight() {
    float outerCone = 0.90f;
    float innerCone = 0.95f;

    // ambient light
    float ambient = 0.20f;
    
    // diffuse light
    vec3 normal = normalize(v_normal);
    vec3 lightDirection = normalize(lightPos - crntPos);
    float diffuse = max(dot(normal, lightDirection), 0.0f);

    //specular light blinn-phong
    float specular = 0.0f;
    if (diffuse != 0.0f) // Only calculate if there is diffuse light
    {
        vec3 viewDirection = normalize(camPos - crntPos);
        vec3 reflectionDirection = reflect(-lightDirection, normal);
        vec3 halfwayVec = normalize(lightDirection + viewDirection);
        float specAmount = pow(max(dot(normal, halfwayVec), 0.0f), 16);
        specular = specAmount * u_SpecularStrength;
    }

    float angle = dot(vec3(0.0f, -1.0f, 0.0f), -lightDirection);
    float inten = clamp((angle - outerCone) / (innerCone - outerCone), 0.0f, 1.0f);

    vec4 texColor = useTexture ? texture(diffuse0, v_TexCoord) * baseColorFactor : baseColorFactor;
    //vec4 texColor = texture(diffi)
    //vec4 texColor = texture(diffuse0, v_TexCoord);
    float specMap = texture(specular0, v_TexCoord).r;
    vec4 finalColor = (texColor * (diffuse * inten + ambient) + specMap * specular * inten) * lightColor;

    return vec4(finalColor.rgb, texColor.a); // Preserve alpha
}

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
    float depth = logisticDepth(gl_FragCoord.z, 0.2f, 100.0f);
    vec4 directLightColor = directLight();  // Separate color and alpha
    vec3 depthColor = (1.0f - depth) + depth * u_BackgroundColor;
    vec3 finalColor = directLightColor.rgb * depthColor;//(1.0f - depth) + depth * u_BackgroundColor;
    
    // Preserve the alpha from directLight()
    //fragColor = vec4(finalColor, directLightColor.a);
    //test shadowMap
    //fragColor = vec4(texture(finalColor, v_TexCoord).xyz, 1.0f);
    fragColor = vec4(finalColor, directLightColor.a); // fragColor is the fragment in the framebuffer
}