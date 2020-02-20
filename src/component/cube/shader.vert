#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 tex;

//layout(location = 0) out vec4 frag_color;
layout(location = 0) out vec2 tex_out;

layout(binding = 0, std140) uniform PushConstants {
//layout(push_constant, std140) uniform PushConstants {
    mat4 transform;
    //float t;
} push;


void main() {
    //frag_color = vec4(1.0, 0.0, 0.0, 1.0);
    tex_out = tex;

    //gl_Position = push.transform * vec4(pos + sin(), 1.0);
    gl_Position = push.transform * vec4(pos, 1.0);
    //gl_Position = vec4(pos, 1.0);
}
