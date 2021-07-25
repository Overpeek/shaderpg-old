#version 420

layout (location = 0) in
vec2 in_pos;

layout (location = 0) out
vec2 out_pos;



void main() {
	gl_Position = vec4(in_pos, 0.0, 1.0);
	out_pos = (in_pos + vec2(1.0, 1.0)) / 2.0;
}