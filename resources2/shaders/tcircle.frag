#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform U {
    vec4 c;
    float r;
    float w;
} u;

void main() {
    vec3 c = u.c.rgb;

    vec2 xy = (tex - 0.5) * vec2(1920.0, 1080.0);

    float r = 200.0 * (u.r + 1.0);
    float delta = 1.0;
    float d = distance(xy, vec2(0.0));

    float sw = 16.0 * (u.w + 1.0);

    float a = smoothstep(r - delta, r + delta, d) - smoothstep(r + sw - delta, r + sw + delta, d);

    //float delta = fwidth(r);
    //float a = 1.0 - smoothstep(1.0 - delta, 1.0 + delta, r);

    color = vec4(c, a - 0.8);
}
