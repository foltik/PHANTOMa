#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform texture2D img1;
layout(set = 0, binding = 1) uniform texture2D img2;
layout(set = 0, binding = 2) uniform sampler samp;

void main() {
    vec4 c1 = texture(sampler2D(img1, samp), tex);
    vec4 c2 = texture(sampler2D(img2, samp), tex);
    color = c1 + c2;
}
