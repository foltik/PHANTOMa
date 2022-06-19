#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec4 pos;
layout(location = 1) in vec2 tex;

layout(location = 0) out vec2 otex;

layout(set = 0, binding = 0, std140) uniform ModelTransform {
    mat4 m;
} model;

void main() {
    otex = tex;
    gl_Position = pos;
}
