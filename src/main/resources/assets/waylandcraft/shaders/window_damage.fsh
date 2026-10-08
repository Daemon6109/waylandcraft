#version 330
#extension GL_ARB_separate_shader_objects : require

layout(location = 0) out vec4 fragColor;

void main() {
	vec3 color = vec3(1.0, 0.0, 0.0);
	float alpha = 0.5;
	fragColor = vec4(color * alpha, alpha); // Have to do premultiplied alpha here
}
