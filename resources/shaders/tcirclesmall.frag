#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

void main() {
    vec3 c = vec3(1.0);

    vec2 xy = (tex - 0.5) * vec2(1920.0, 1080.0);

    float r = 50.0;
    float delta = 1.0;
    float d = distance(xy, vec2(0.0));

    float sw = 16.0;

    float a = smoothstep(r - delta, r + delta, d) - smoothstep(r + sw - delta, r + sw + delta, d);

    color = vec4(c, a - 0.8);
}
