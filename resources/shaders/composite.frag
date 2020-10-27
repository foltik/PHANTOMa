#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_nonuniform_qualifier : require

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform texture2D imgs[];
layout(set = 0, binding = 1) uniform sampler samp;
layout(set = 0, binding = 2) uniform U {
    uint n;
} u;

void main() {
    vec3 c = vec3(0.0);
    for  (int i = 0; i < u.n; i++)
        c += texture(sampler2D(imgs[i], samp), tex).rgb;

    color = vec4(c, 1.0);
}
