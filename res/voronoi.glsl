#version 420
#include<rand>

layout(location = 0) out vec4 color;

layout(location = 0) in vec2 position;

layout(binding = 0) uniform UBO {
	float time;
} ubo;

vec3 voronoi(vec2 uv) {
	vec2 id = floor(uv);
	vec2 pos = fract(uv);

	float min_dist = 2.0;
	vec2 min_id = vec2(0, 0);

	for ( int y = -1; y < 2; y++ ) { for ( int x = -1; x < 2; x++ ) {
		vec2 id_off = vec2(x, y);
		vec2 id_l = id + vec2(50.5) * floor(ubo.time + 0) + id_off;
		vec2 id_r = id + vec2(50.5) * floor(ubo.time + 1) + id_off;
		vec2 rand_pos_l = vec2(rand(id_l), rand(vec2(400000) + id_l));
		vec2 rand_pos_r = vec2(rand(id_r), rand(vec2(400000) + id_r));

		vec2 rand_pos = mix(rand_pos_l, rand_pos_r, fract(ubo.time));
		float dist = length(pos - id_off - rand_pos);

		if (dist < min_dist) {
			min_dist = dist;
			min_id = id + id_off;
		}
	}}

	return vec3(min_dist, min_id);
}

void main() {
	vec2 uv = position * 5.0;
	vec3 r = voronoi(uv);
	float dist = r.r;
	vec2 id = r.gb;

	vec3 col = vec3(rand(id), rand(id + vec2(5000)), rand(id + vec2(500)));
	color = vec4(col, 1.0);
}