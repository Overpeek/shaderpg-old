#version 420

layout(location = 0) out vec4 color;

layout(location = 0) in vec2 position;

layout(binding = 0) uniform UBO {
	float time;
	float aspect;
} ubo;



void main() {
	float phase = 2.0943951; // 2/3 pi

	color = vec4(
		sin(ubo.time * 3.0 + phase * 0 + position.x * ubo.aspect * 2.0) * 0.5 + 0.5,
		sin(ubo.time * 3.0 + phase * 1 + position.y * 2.0) * 0.5 + 0.5,
		sin(ubo.time * 3.0 + phase * 2) * 0.5 + 0.5,
		1.0
	);
}