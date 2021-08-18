#version 420

#include <rand>
const float EPSILON = 0.001;
const vec3 LIGHT = normalize(vec3(0.5, -1.0, 1.0));

layout(location = 0) out vec4 color;

layout(location = 0) in vec2 position;

layout(binding = 0) uniform UBO {
	float time;
	float aspect;
} ubo;



struct Ray {
	vec3 pos;
	vec3 dir;
	bool hit;
};



float rand3(vec3 p) {
	return rand(vec2(rand(p.xy), p.z));
}

vec3 rand3to3(vec3 p) {
	return vec3(rand3(vec3(-400000) + p), rand3(p), rand3(vec3(400000) + p));
}

vec4 voronoi(vec3 uv) {
	vec3 id = floor(uv);
	vec3 pos = fract(uv);

	float min_dist = 2.0;
	vec3 min_id = vec3(0, 0, 0);

	for (int y = -1; y < 2; y++) { for (int x = -1; x < 2; x++) { for (int z = -1; z < 2; z++) {
		vec3 id_off = vec3(x, y, z);
		vec3 id_l = id + vec3(50.5) * floor(ubo.time + 0) + id_off;
		vec3 id_r = id + vec3(50.5) * floor(ubo.time + 1) + id_off;
		vec3 rand_pos_l = rand3to3(id_l);
		vec3 rand_pos_r = rand3to3(id_r);

		vec3 rand_pos = mix(rand_pos_l, rand_pos_r, fract(ubo.time));
		float dist = length(pos - id_off - rand_pos);

		if (dist < min_dist) {
			min_dist = dist;
			min_id = id + id_off;
		}
	}}}

	return vec4(min_dist, min_id);
}

float sdBox(vec3 point, vec3 box) {
	vec3 dist = abs(point) - box;
	return min(max(dist.x, max(dist.y, dist.z)), 0.0) + length(max(dist, 0.0));
}

float scene(vec3 point) {
	return sdBox(point - vec3(0.2, 0.0, 0.0), vec3(0.4, 0.6, 0.8));
}

vec3 normal(vec3 point) {
    float xPl = scene(vec3(point.x + EPSILON, point.y, point.z));
    float xMi = scene(vec3(point.x - EPSILON, point.y, point.z));
    float yPl = scene(vec3(point.x, point.y + EPSILON, point.z));
    float yMi = scene(vec3(point.x, point.y - EPSILON, point.z));
    float zPl = scene(vec3(point.x, point.y, point.z + EPSILON));
    float zMi = scene(vec3(point.x, point.y, point.z - EPSILON));
    
	float xDiff = xPl - xMi;
    float yDiff = yPl - yMi;
    float zDiff = zPl - zMi;
    
	return normalize(vec3(xDiff, yDiff, zDiff));
}

vec3 texture_map(vec3 point) {
	vec4 v = voronoi(point * 4.0);
	float dist = v.x;
	vec3 id = v.yzw;

	return rand3to3(id);
}

Ray march(Ray ray) {
	for (int i = 0; i < 256; i++) {
		float dist = sdBox(ray.pos - vec3(0.2, 0.0, 0.0), vec3(0.4, 0.6, 0.8));

		if (dist < EPSILON) {
			ray.hit = true;
			break;
		}

		ray.pos += ray.dir * dist;
	}

	return ray;
}

Ray camera(vec2 uv, vec3 camera_pos, vec3 look_at, float fov){
    vec3 f = normalize(look_at - camera_pos);
    vec3 r = cross(vec3(0.0, 1.0, 0.0), f);
    vec3 u = cross(f, r);
	
	vec3 c = camera_pos + f * fov;
    vec3 i = c + uv.x * r + uv.y * u;
    vec3 dir = i - camera_pos;
    
	return Ray(camera_pos, dir, false);
}

void main() {
	vec2 aspect = vec2(ubo.aspect, 1.0);
	vec2 uv = position * aspect * 2.0 - aspect;

	Ray ray = camera(
		uv,
		vec3(cos(ubo.time) * 1.4, -1.0, sin(ubo.time) * 1.4),
		vec3(0.0, 0.0, 0.0),
		0.7
	);
	ray = march(ray);
	
	vec3 col = vec3(0.0, 0.0, 0.0);
	if (ray.hit) {
		col = texture_map(ray.pos);
		col *= dot(normal(ray.pos), LIGHT) * 0.5 + 0.5;
	}

	color = vec4(col, 1.0);
}