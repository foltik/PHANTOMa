#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_nonuniform_qualifier : require

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform U {
    float t;
} u;

void main() {
    vec2 st = vec2(tex.x, tex.y);

    float t = u.t;
    vec2 p = vec2(sin(t), cos(t) * abs(cos(t)));
	float dv = 1.0 / 1080.0;

    float theta = 0.7 * abs(p.y);
    float r = distance(st, vec2(0.5, 0.5));
    float ri = 0.35;
    float ro = 0.4;
    
    vec2 uv = st - vec2(0.5, 0.5);
    float cos_a = dot(uv, p) / (length(uv) * length(p));
    
    float fr_angle = smoothstep(ri - dv, ri + dv, r) * smoothstep(r - dv, r + dv, ro);
    float fr_dist = smoothstep(cos_a - dv, cos_a + dv, theta) * smoothstep(-theta - dv, -theta + dv, cos_a);
    
    color = vec4(vec3(fr_angle * fr_dist), 1.0);
}
