#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 tex;
layout(location = 2) in vec3 norm;

layout(location = 0) out vec2 otex;
layout(location = 1) out vec3 onorm;
layout(location = 2) out vec3 opos;

layout(set = 1, binding = 0) uniform Camera {
    mat4 view;
    mat4 proj;
} cam;

layout(set = 2, binding = 0) uniform Model {
    mat4 transform;
} m;

layout(set = 3, binding = 0) uniform Object {
    mat4 transform;
} o;

void main() {
    otex = tex;
    onorm = norm;
    opos = pos;

    gl_Position = cam.proj * cam.view * o.transform * m.transform * vec4(pos, 1.0);
}
