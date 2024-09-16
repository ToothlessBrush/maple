#version 330 core

out vec4 color;

in vec3 crntPos;
in vec3 v_normal;
in vec4 v_Color;
in vec2 v_TexCoord;

uniform sampler2D diffuse0;
uniform sampler2D specular0;

uniform vec4 lightColor;
uniform vec3 lightPos;
uniform vec3 camPos;

vec4 pointLight() {
	vec3 lightVec = lightPos - crntPos;
	float dist = length(lightVec);
	float a = 1.0f;
	float b = 0.04f;
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

	
	vec4 texColor = texture(diffuse0, v_TexCoord);
	float specMap = texture(specular0, v_TexCoord).r;
	return (texColor * (diffuse * inten + ambient) + specMap * specular * inten) * lightColor;
}

vec4 directLight() {
	// ambient light
	float ambient = 0.20f;
	
	// diffuse light
	vec3 normal = normalize(v_normal);
	vec3 lightDirection = normalize(vec3(1.0f, 1.0f, 0.0f));
	float diffuse = max(dot(normal, lightDirection), 0.0f);

	// specular light
	float specularLight = 0.50f;
	vec3 viewDirection = normalize(camPos - crntPos);
	vec3 reflectionDirection = reflect(-lightDirection, normal);
	float specAmount = pow(max(dot(viewDirection, reflectionDirection), 0.0f), 16);
	float specular = specAmount * specularLight;

	
	vec4 texColor = texture(diffuse0, v_TexCoord);
	float specMap = texture(specular0, v_TexCoord).r;
	return (texColor * (diffuse + ambient) + specMap * specular) * lightColor;
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

	
	vec4 texColor = texture(diffuse0, v_TexCoord);
	float specMap = texture(specular0, v_TexCoord).r;
	return (texColor * (diffuse * inten + ambient) + specMap * specular * inten) * lightColor;
}



void main() {
	color = vec4(directLight().rgb, 1.0);
}