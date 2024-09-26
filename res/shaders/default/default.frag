#version 330 core

out vec4 fragColor;

in vec3 crntPos;
in vec3 v_normal;
in vec4 v_Color;
in vec2 v_TexCoord;

uniform sampler2D diffuse0;
uniform sampler2D specular0;

uniform vec4 baseColorFactor;

uniform bool useTexture;

uniform vec4 lightColor;
uniform vec3 lightPos;
uniform vec3 camPos;

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

    // specular light
    float specularLight = 0.50f;
    vec3 viewDirection = normalize(camPos - crntPos);
    vec3 reflectionDirection = reflect(-lightDirection, normal);
    float specAmount = pow(max(dot(viewDirection, reflectionDirection), 0.0f), 16);
    float specular = specAmount * specularLight;


    
    vec4 texColor = useTexture ? texture(diffuse0, v_TexCoord) : baseColorFactor;
    //vec4 texColor = texture(diffuse0, v_TexCoord);
    float specMap = texture(specular0, v_TexCoord).r;
    vec4 finalColor =  (texColor * (diffuse * inten + ambient) + specMap * specular * inten) * lightColor;

    return vec4(finalColor.rgb, texColor.a); // Preserve alpha
}

vec4 directLight() {
    // Ambient light

    
    float ambient = 0.20f;
    
    // Diffuse light
    vec3 normal = normalize(v_normal);
    vec3 lightDirection = normalize(vec3(1.0f, 1.0f, 0.0f)); // Directional light
    float diffuse = max(dot(normal, lightDirection), 0.0f);

    // Specular light
    float specularLight = 0.50f;
    vec3 viewDirection = normalize(camPos - crntPos);
    vec3 reflectionDirection = reflect(-lightDirection, normal);
    float specAmount = pow(max(dot(viewDirection, reflectionDirection), 0.0f), 16);
    float specular = specAmount * specularLight;

    vec4 texColor = useTexture ? texture(specular0, v_TexCoord) : baseColorFactor;
    //vec4 texColor = texture(diffuse0, v_TexCoord);
    float specMap = texture(specular0, v_TexCoord).g;

    // Combine textures with lighting
    vec4 finalColor = (texColor * (diffuse + ambient) + specMap * specular) * lightColor;

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

    // specular light
    float specularLight = 0.50f;
    vec3 viewDirection = normalize(camPos - crntPos);
    vec3 reflectionDirection = reflect(-lightDirection, normal);
    float specAmount = pow(max(dot(viewDirection, reflectionDirection), 0.0f), 16);
    float specular = specAmount * specularLight;

    float angle = dot(vec3(0.0f, -1.0f, 0.0f), -lightDirection);
    float inten = clamp((angle - outerCone) / (innerCone - outerCone), 0.0f, 1.0f);

    vec4 texColor = useTexture ? texture(diffuse0, v_TexCoord) * baseColorFactor : baseColorFactor;
    //vec4 texColor = texture(diffuse0, v_TexCoord);
    float specMap = texture(specular0, v_TexCoord).r;
    vec4 finalColor = (texColor * (diffuse * inten + ambient) + specMap * specular * inten) * lightColor;

    return vec4(finalColor.rgb, texColor.a); // Preserve alpha
}

float near = 0.1f;
float far = 1000.0f;

float linearizeDepth(float depth) {
    return (2.0f * near * far) / (far + near - (depth * 2.0 - 1.0) * (far - near));
}

float logisticDepth(float depth, float steepness, float offset) {
    float zVal = linearizeDepth(depth);
    return (1 / (1 + exp(-steepness * (zVal - offset))));
}

void main() {
    float depth = logisticDepth(gl_FragCoord.z, 0.2f, 900.0f);
    vec4 directLightColor = directLight();  // Separate color and alpha
    vec3 finalColor = directLightColor.rgb * (1.0f - depth) + vec3(depth * 0.85f, depth * 0.85f, depth * 0.90f);
    
    // Preserve the alpha from directLight()
    fragColor = vec4(finalColor, directLightColor.a);
}