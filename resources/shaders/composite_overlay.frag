#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_nonuniform_qualifier : require

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform texture2D imgs[];
layout(set = 0, binding = 1) uniform sampler samp;

void main() {
    vec2 st = vec2(tex.x, tex.y);

    vec3 c0 = texture(sampler2D(imgs[0], samp), st).rgb;
    vec3 c1 = texture(sampler2D(imgs[1], samp), st).rgb;

    if (dot(c1, c1) != 0) {
        color = vec4(c1, 1.0);
    } else {
        color = vec4(c0, 1.0);
    }
}
