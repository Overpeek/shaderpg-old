#version 420
#extension GL_ARB_separate_shader_objects : enable



#[gears_bindgen(uniform)]
struct UBO {
	float time;
} ubo;

#[gears_bindgen(in)]
struct VertexData {
	vec2 pos;
} vert_in;



void main() {
	gl_Position = vec4(vert_in.pos, 0.0, 1.0);
}