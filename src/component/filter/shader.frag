#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(binding = 0) uniform sampler2D img;

void main() {
    vec4 c = texture(img, tex);

    float d = 0.003;

    float r = texture(img, tex + vec2(0.0, -d)).r;
    float g = texture(img, tex + vec2(-d, d)).g;
    float b = texture(img, tex + vec2(d, -d)).b;

    color = vec4(r, g, b, 1.0);
}
