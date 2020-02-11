#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 pos;
layout(location = 0) out vec4 frag_color;


layout(push_constant) uniform PushConstants {
    mat4 transform;
    //uint frame;
} push_constants;


void main() {
    frag_color = vec4(1.0, 0.0, 0.0, 1.0);

    //gl_Position = push_constants.transform * vec4(pos + sin(push_constants.frame / 60.0), 1.0);
    gl_Position = push_constants.transform * vec4(pos, 1.0);
    //gl_Position = vec4(pos, 1.0);
}
