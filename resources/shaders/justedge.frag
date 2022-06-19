#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform texture2D imgs[];
layout(set = 0, binding = 1) uniform sampler samp;

layout(set = 1, binding = 0) uniform U {
    float w;
    float h;
    float amt;
} u;

void main() {
    vec2 res = vec2(u.w, u.h);
    vec2 st = vec2(tex.x, tex.y);

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

    color = vec4(mix(i, c, u.amt), 1.0);
}
