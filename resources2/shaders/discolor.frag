#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform texture2D img;
layout(set = 0, binding = 1) uniform sampler samp;
layout(set = 0, binding = 2) uniform Uniforms {
    float t;
} u;

void main() {
    vec4 c = texture(sampler2D(img, samp), tex);
    c.rg += tex * ((sin(u.t) + 1.0) / 2.0);
    color = c;
}
