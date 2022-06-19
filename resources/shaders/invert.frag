#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform texture2D imgs[];
layout(set = 0, binding = 1) uniform sampler samp;

layout(set = 1, binding = 0) uniform Uniforms {
    float fr;
} u;

void main() {
    vec3 img = texture(sampler2D(imgs[0], samp), tex).rgb;
    color = vec4(mix(img, vec3(1.0) - img, u.fr), u.fr);
}
