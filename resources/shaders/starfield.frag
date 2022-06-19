#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_nonuniform_qualifier : require

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform U {
    vec3 c;
    float x;
    float y;
    float w;
    float h;
    float t;
    float speed;
    float warp;
    float acid;
} u;

void main() {
    vec2 st = vec2(tex.x, tex.y);
    float t = u.t * u.speed;

    vec2 res = vec2(u.w, u.h);
    vec2 uv = st * res;
    uv -= vec2(u.x, u.y) * res;
    uv /= 100.0 + ((1.0 - u.warp) * 2000.0);

    float b = ceil(atan(uv.x, uv.y) * 6e2);
    float h = cos(b);
    float z = h / dot(uv, uv);
    float a = -100.0 * (1.0 - u.acid);
    float p = exp(fract(z + h * b + t) * (a - 2.0)) / z;

    color = vec4(u.c * vec3(p), 1.0);
}
