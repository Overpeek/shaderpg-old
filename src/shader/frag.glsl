#version 420
#extension GL_ARB_separate_shader_objects : enable



layout(location = 0) out vec4 color;

layout(location = 0) in vec2 position;

layout(binding = 0) uniform UBO {
	float time;
} ubo;



void main() {
	float phase = 2.0943951; // 2/3 pi

	color = vec4(
		sin(ubo.time + phase * 0 + position.x) * 0.5 + 0.5,
		sin(ubo.time + phase * 1 + position.y) * 0.5 + 0.5,
		sin(ubo.time + phase * 2) * 0.5 + 0.5,
		1.0
	);
}