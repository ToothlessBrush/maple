#version 330 core

out vec4 color;

in vec4 v_Color;
in vec2 v_TexCoord;

in vec3 v_normal;
in vec3 crntPos;

uniform sampler2D u_Texture;

uniform vec4 lightColor;
uniform vec3 lightPos;
uniform vec3 camPos;

void main() {
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
	float specAmount = pow(max(dot(viewDirection, reflectionDirection), 0.0f), 8);
	float specular = specAmount * specularLight;

	
	vec4 texColor = texture(u_Texture, v_TexCoord);
	color = texColor * lightColor * (diffuse + ambient + specular);
}