#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 pos;
layout(location = 0) out vec4 frag_color;

layout(push_constant) uniform PushConstants {
    float t;
} push_constants;

void main() {
    frag_color = vec4(1.0, 0.0, 0.0, 1.0);
    gl_Position = vec4(pos + push_constants.t, 1.0);
}
