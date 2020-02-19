#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) out vec2 tex_out;

void main() {
    const vec4 vertices[3] = vec4[3](vec4( 0.25, -0.25, 0, 1.0),
    vec4(-0.25, -0.25, 0, 1.0),
    vec4( 0.25, 0.25, 0, 1.0));

    gl_Position = vertices[gl_VertexIndex];
}
