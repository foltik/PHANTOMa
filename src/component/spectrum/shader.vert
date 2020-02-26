#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 tex;

layout(location = 0) out vec2 tex_out;

layout(binding = 0, std140) uniform PushConstants {
    mat4 transform;
} push;


void main() {
    tex_out = tex;

    gl_Position = push.transform * vec4(pos + gl_InstanceIndex * 0.05, 1.0);
}
