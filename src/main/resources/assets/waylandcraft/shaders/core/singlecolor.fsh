#version 330
#extension GL_ARB_separate_shader_objects : require

layout(location = 0) out vec4 Color;

void main() {
	Color = vec4(RED, GREEN, BLUE, ALPHA);
}
