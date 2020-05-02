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

void main() {
    mat4 model = inverse(u.view);

    //tex = (pos.xy / 2.0) + 0.5;
    tex = itex;
    norm = mat3(transpose(inverse(model))) * inorm;
    pos = vec3(model * vec4(ipos, 1.0));
    gl_Position = u.proj * u.view * vec4(ipos, 1.0);

    //tex = pos.xy;
    //gl_Position = vec4(pos.xy, 0.0, 0.0);

    // billboard
    //tex = vec2((gl_VertexIndex << 1) & 2, gl_VertexIndex & 2);
    //gl_Position = vec4(tex * 2.0 - 1.0, 0.0, 1.0);
}
