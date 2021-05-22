#version 420
#extension GL_ARB_separate_shader_objects : enable



#[gears_bindgen(in)]
struct VertexData {
	vec2 pos;
} vert_in;

#[gears_bindgen(out)]
struct Shared {
	vec2 pos;
} vert_out;



void main() {
	gl_Position = vec4(vert_in.pos, 0.0, 1.0);
	vert_out.pos = (vert_in.pos + vec2(1.0, 1.0)) / 2.0;
}