#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 tex;

layout(location = 0) out vec2 tex_out;

layout(binding = 0, std140) uniform Uniforms {
    float n;
    float fft[256];
} u;

void main() {
    tex_out = tex;

    vec3 p = pos;

    if (gl_VertexIndex == 0 || gl_VertexIndex == 1 || gl_VertexIndex == 5)
        p.y -= u.fft[gl_InstanceIndex] * 1.8 * 4;

    p.x += (gl_InstanceIndex * (1.8 / u.n)) - 0.9;
    p.y += 0.9;

    gl_Position = vec4(p, 1.0);
}
