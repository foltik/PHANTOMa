#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

void main() {
    vec2 st = vec2(tex.x, tex.y);
    color = vec4(st, 0.0, 1.0);
}
