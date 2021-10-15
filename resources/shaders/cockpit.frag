#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform U {
    float f0;
    float f1;
    float f2;
    float beat;
} u;

vec3 rect(vec2 st, vec2 pos, float w, float h, vec3 c) {
    pos = pos / vec2(1920.0, 1080.0);

    vec2 bl = pos;
    vec2 tr = vec2(pos.x + w, pos.y + h);

    vec2 region = step(bl, st) * (1.0 - step(tr, st));
    vec2 fill = region * (region.x * region.y);

    return c * vec3(fill.x + fill.y);
}

void main() {
    vec3 r0 = rect(tex, vec2(878.0, 153.0), 0.016, u.f0 * 90.0 / 1080.0, vec3(0.0, 1.0, 0.0));
    vec3 r1 = rect(tex, vec2(948.5, 153.0), 0.016, u.f1 * 230.0 / 1080.0, vec3(0.0, 1.0, 0.0));
    vec3 r2 = rect(tex, vec2(1019.0, 153.0), 0.016, u.f2 * 150.0 / 1080.0, vec3(0.0, 1.0, 0.0));

    float w = u.beat * 96.7;
    float h = u.beat * 24.0;
    vec2 cent = vec2(950.0 - 0.75 * w, 122.0 - (h / 2.0));
    
    vec3 r3 = rect(tex, cent, w / 1080.0, h / 1080.0, vec3(0.0, 1.0, 0.0));

    vec3 c = r0 + r1 + r2 + r3;

    if (dot(c, c) != 0) {
        color = vec4(c, 1.0);
    } else {
        color = vec4(0.0);
    }
}
