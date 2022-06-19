#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 tex;
layout(location = 2) in vec3 norm;

layout(location = 0) out vec3 opos;
layout(location = 1) out vec2 otex;
layout(location = 2) out vec3 onorm;

layout(set = 0, binding = 0, std140) uniform CameraView {
    mat4 m;
} view;

layout(set = 0, binding = 1, std140) uniform CameraProj {
    mat4 m;
} proj;

layout(set = 3, binding = 0, std140) uniform ModelTransform {
    mat4 m;
} model;

void main() {
    mat4 model_view = view.m * model.m;
    mat4 mnormal = transpose(inverse(model_view));

    opos = vec3(model_view * vec4(pos, 1.0));
    otex = tex;
    onorm = vec3(mnormal * vec4(norm, 1.0));
    gl_Position = proj.m * model_view * vec4(pos, 1.0);
}
