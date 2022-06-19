#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform texture2D imgs[];
layout(set = 0, binding = 1) uniform sampler samp;

layout(set = 1, binding = 0) uniform Uniforms {
    float t;
    float tc;
    float pause;
    float glitch;
    float glitch_mo;
    float vhs;
    float red;
    float flash;
    float shake;
    float black;
    float edge;
    float mega;
} u;

float inside(float x, float lo, float hi) {
    return step(lo, x) - step(hi, x);
}

float rand(vec2 p) {
    float dt = dot(p, vec2(12.9898, 78.233));
    float sn = mod(dt, 3.14);
    return fract(sin(sn) * 43758.5453);
}

float rrand(vec2 p, float lo, float hi) {
    return lo + rand(p) * (hi - lo);
}

vec3 permute(vec3 x) { return mod(((x*34.0)+1.0)*x, 289.0); }
float snoise(vec2 p) {
    const vec4 C = vec4(0.211324865405187, 0.366025403784439, -0.577350269189626, 0.024390243902439);
    vec2 i = floor(p + dot(p, C.yy));
    vec2 x0 = p - i + dot(i, C.xx);
    vec2 i1 = (x0.x > x0.y) ? vec2(1.0, 0.0) : vec2(0.0, 1.0);
    vec4 x12 = x0.xyxy + C.xxzz;
    x12.xy -= i1;
    i = mod(i, 289.0);
    vec3 pm = permute(permute(i.y + vec3(0.0, i1.y, 1.0)) + i.x + vec3(0.0, i1.x, 1.0 ));
    vec3 m = max(0.5 - vec3(dot(x0, x0), dot(x12.xy, x12.xy), dot(x12.zw, x12.zw)), 0.0);
    m = m*m;
    m = m*m;
    vec3 x = 2.0 * fract(pm * C.www) - 1.0;
    vec3 h = abs(x) - 0.5;
    vec3 ox = floor(x + 0.5);
    vec3 a0 = x - ox;
    m *= 1.79284291400159 - 0.85373472095314 * (a0 * a0 + h * h);
    vec3 g;
    g.x  = a0.x  * x0.x  + h.x  * x0.y;
    g.yz = a0.yz * x12.xz + h.yz * x12.yw;
    return 130.0 * dot(m, g);
}

void main() {
    vec2 res = vec2(1920.0, 1080.0);
    vec2 st = vec2(tex.x, 1.0 - tex.y);

    vec3 i = texture(sampler2D(imgs[0], samp), st).rgb;

    vec3 TL = texture(sampler2D(imgs[0], samp), st + vec2(-1.0, 1.0) / res).rgb;
    vec3 TM = texture(sampler2D(imgs[0], samp), st + vec2(0.0, 1.0) / res).rgb;
    vec3 TR = texture(sampler2D(imgs[0], samp), st + vec2(1.0, 1.0) / res).rgb;

    vec3 ML = texture(sampler2D(imgs[0], samp), st + vec2(-1.0, 0.0) / res).rgb;
    vec3 MR = texture(sampler2D(imgs[0], samp), st + vec2(1.0, 0.0) / res).rgb;
    
    vec3 BL = texture(sampler2D(imgs[0], samp), st + vec2(-1.0, -1.0) / res).rgb;
    vec3 BM = texture(sampler2D(imgs[0], samp), st + vec2(0.0, -1.0) / res).rgb;
    vec3 BR = texture(sampler2D(imgs[0], samp), st + vec2(1.0, -1.0) / res).rgb;

    vec3 GradX = -TL + TR - 2.0 * ML + 2.0 * MR - BL + BR;
    vec3 GradY = TL + 2.0 * TM + TR - BL - 2.0 * BM - BR;

    /* vec2 gradCombo = vec2(GradX.r, GradY.r) + vec2(GradX.g, GradY.g) + vec2(GradX.b, GradY.b);
    fragColor = vec4(gradCombo.r, gradCombo.g, 0, 1);*/

    float r = length(vec2(GradX.r, GradY.r));
    float g = length(vec2(GradX.g, GradY.g));
    float b = length(vec2(GradX.b, GradY.b));

    vec3 c = vec3(r, g, b);

    color = vec4(mix(i, c, min(u.edge + u.mega, 1.0)), 1.0);
}
