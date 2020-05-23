#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 ipos;
layout(location = 1) in vec2 itex;
layout(location = 2) in vec3 inorm;

layout(location = 0) out vec2 tex;
layout(location = 1) out vec3 norm;
layout(location = 2) out vec3 pos;

layout(set = 0, binding = 0) uniform Uniforms {
    mat4 view;
    mat4 proj;
} u;

layout(set = 1, binding = 0) uniform Transform {
    mat4 model;
} tr;

void main() {
    tex = itex;
    norm = inorm;
    pos = ipos;
    gl_Position = u.proj * u.view * tr.model * vec4(pos, 1.0);
}
