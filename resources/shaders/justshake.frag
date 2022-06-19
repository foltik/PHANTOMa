#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform texture2D imgs[];
layout(set = 0, binding = 1) uniform sampler samp;

layout(set = 1, binding = 0) uniform U {
    float t;
    float speed;
    float amt;
} u;

float rand(vec2 p) {
    float dt = dot(p, vec2(12.9898, 78.233));
    float sn = mod(dt, 3.14);
    return fract(sin(sn) * 43758.5453);
}

vec2 shake(vec2 uv, vec2 amt, float t) {
    float dx = (rand(vec2(t))     - 0.5) * amt.x;
    float dy = (rand(vec2(2.0*t)) - 0.5) * amt.y;
    return uv + vec2(dx, dy);
}

void main() {
    vec2 st = vec2(tex.x, tex.y);
    vec2 uv = shake(st, vec2(u.amt), u.t * u.speed);
    color = texture(sampler2D(imgs[0], samp), uv);
}
