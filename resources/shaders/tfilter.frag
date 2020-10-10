#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform texture2D img;
layout(set = 0, binding = 1) uniform sampler samp;

layout(set = 1, binding = 0) uniform U {
    vec4 c;
    float r;
    float w;
    float t;
} u;

void main() {
    color = vec4(texture(sampler2D(img, samp), tex).rgb, 1.0);
    color.r += 0.01 * sin(u.t) + 0.01;
}
