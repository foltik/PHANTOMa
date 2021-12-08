#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_nonuniform_qualifier : require

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform U {
    vec3 color;
    float nx;
    float ny;
    float t;
} u;

float rand(vec2 p) {
    float dt = dot(p, vec2(12.9898, 78.233));
    float sn = mod(dt, 3.14);
    return fract(sin(sn) * 43758.5453);
}

void main() {
    vec2 st = vec2(tex.x, 1.0 - tex.y);

    float t = u.t / 4.0;
    vec2 tiles = vec2(u.nx, u.ny);

    vec4 noise = vec4(rand(floor(st * tiles) / tiles));
	float p = 1.0 - mod(noise.r + noise.g + noise.b + t, 1.0);
	p = min(max(p * 3.0 - 1.8, 0.025), 2.0);
	
	vec2 r = mod(st * tiles, 1.0);
	r = vec2(pow(r.x - 0.5, 2.0), pow(r.y - 0.5, 2.0));
	p *= 1.0 - pow(min(1.0, 12.0 * dot(r, r)), 2.0);

    color = vec4(u.color * vec3(p), 1.0);
}
