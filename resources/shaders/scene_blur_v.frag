#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform texture2D img;
layout(set = 0, binding = 1) uniform sampler samp;

vec4 blur9(vec2 amt) {
    vec2 uv0 = tex + vec2(-amt.s, -amt.t);
    vec2 uv1 = tex + vec2(   0.0, -amt.t);
    vec2 uv2 = tex + vec2(+amt.s, -amt.t);
    vec2 uv3 = tex + vec2(-amt.s,    0.0);
    vec2 uv4 = tex + vec2(0.0,       0.0);
    vec2 uv5 = tex + vec2(+amt.s,    0.0);
    vec2 uv6 = tex + vec2(-amt.s, +amt.t);
    vec2 uv7 = tex + vec2(0.0,    +amt.t);
    vec2 uv8 = tex + vec2(+amt.s, +amt.t);

    vec4 col0 = texture(sampler2D(img, samp), uv0);
    vec4 col1 = texture(sampler2D(img, samp), uv1);
    vec4 col2 = texture(sampler2D(img, samp), uv2);
    vec4 col3 = texture(sampler2D(img, samp), uv3);
    vec4 col4 = texture(sampler2D(img, samp), uv4);
    vec4 col5 = texture(sampler2D(img, samp), uv5);
    vec4 col6 = texture(sampler2D(img, samp), uv6);
    vec4 col7 = texture(sampler2D(img, samp), uv7);
    vec4 col8 = texture(sampler2D(img, samp), uv8);

    vec4 sum = (1.0 * col0 + 2.0 * col1 + 1.0 * col2 +
                2.0 * col3 + 4.0 * col4 + 2.0 * col5 +
                1.0 * col6 + 2.0 * col7 + 1.0 * col8) / 16.0;
    return sum;
}

void main() {
    color = blur9(vec2(0.0, 0.0015));
}
