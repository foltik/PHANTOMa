#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform texture2D img;
layout(set = 0, binding = 1) uniform sampler samp;
layout(set = 0, binding = 2) uniform Uniforms {
    float t;
    float pause;
    float glitch;
    float glitch_mo;
    float red;
} u;

float rand(vec2 p) {
    float dt = dot(p, vec2(12.9898, 78.233));
    float sn = mod(dt, 3.14);
    return fract(sin(sn) * 43758.5453);
}

float rrand(vec2 p, float lo, float hi) {
    return lo + rand(p) * (hi - lo);
}

void main() {
    vec3 c = vec3(0.0);

    vec2 xy = (tex - 0.5) * vec2(1920.0, 1080.0);

    float r = 300.0;
    float delta = 1.0;
    float d = distance(xy, vec2(0.0));

    float sw = 2.0;

    float a = smoothstep(r - delta, r + delta, d) - smoothstep(r + sw - delta, r + sw + delta, d);

    //float r = 0.1;
    //float delta = fwidth(r);
    //float a = 1.0 - smoothstep(1.0 - delta, 1.0 + delta, r);

    color = vec4(1.0, 1.0, 1.0, a - 0.8);
}
