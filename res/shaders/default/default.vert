#version 330 core

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec4 color;
layout(location = 3) in vec2 texCoord;

out vec3 crntPos;
out vec3 v_normal;
out vec4 v_Color;
out vec2 v_TexCoord;
out vec4 fragPosLight;

uniform mat4 u_VP;
uniform mat4 u_Model;

uniform mat4 u_lightSpaceMatrix;

void main() {

	mat4 normalMatrix = transpose(inverse(u_Model));

	//outputs world position of vertices
	crntPos = vec3(u_Model * vec4(position, 1.0f));
	
	// outputs screen position of vertices
	gl_Position = u_VP * u_Model * vec4(position, 1.0); // the 2d screen position in the range of 0 to 1 
	fragPosLight = u_lightSpaceMatrix * vec4(crntPos, 1.0); // the 2d light position in the range of 0 to 1
	v_Color = color;
	v_TexCoord = texCoord;

	//v_normal = normal;

	// apply model matrix to normals to have consistent lighting
	v_normal = normalize((normalMatrix * vec4(normal, 0.0)).xyz);
}