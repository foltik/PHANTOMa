#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform texture2D img;
layout(set = 0, binding = 1) uniform sampler samp;
layout(set = 0, binding = 2) uniform Uniform {
    vec2 pos;
    vec2 scale;
} u;

void main() {
    vec2 tl = step(u.pos, tex);
    vec2 br = step(1.0 - u.pos - u.scale, 1.0 - tex);
    float amt = tl.x * tl.y * br.x * br.y;

    vec2 uv = (tex + u.pos) / u.scale;
    // color = mix(vec4(0.0), vec4(uv, 0.0, 1.0), amt);
    color = mix(vec4(0.0), texture(sampler2D(img, samp), uv), amt);
}
