#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 1) in vec3 norm;
layout(location = 2) in vec3 pos;

layout(location = 0) out vec4 color;

layout(set = 1, binding = 1) uniform Camera {
    vec4 eye;
} cam;

layout(set = 3, binding = 1) uniform Material {
    vec4 ambient;
    vec4 diffuse;
    vec4 specular;
    vec4 emissive;
    vec4 extra;
} mat;
layout(set = 3, binding = 2) uniform sampler samp;
layout(set = 3, binding = 3) uniform texture2D img;


void main() {
    if (dot(mat.emissive, vec4(1.0)) > 0.0) {
        // Texture emissive map?
        color = vec4(mat.emissive.rgb, 1.0);
    } else {
        color = vec4(vec3(0.0), 1.0);
    }
}