#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(binding = 0, std140) uniform Uniforms {
    double t;
    uint w;
    uint h;
} u;

void main() {
    color = vec4(tex, 0, 1);
}
