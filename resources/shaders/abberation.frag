#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform texture2D imgs[];
layout(set = 0, binding = 1) uniform sampler samp;

layout(set = 1, binding = 0) uniform Uniforms {
    float t;
    float fr;
} u;

float hash11(float n) {
    return fract(sin(n) * 43758.5453123);
}

void main() {
    vec2 uv = vec2(tex.x, tex.y);

    float t = abs(sin(u.t));
    float off = u.fr * (.6*(1.0 - t) + 0.08*t*hash11(u.t) + .1);
    float r = texture(sampler2D(imgs[0], samp), .03 * off + uv).r;
    float g = texture(sampler2D(imgs[0], samp), .04 * off + uv).g;
    float b = texture(sampler2D(imgs[0], samp), .05 * off + uv).b;
    
    color = vec4(r, g, b, 1.0);
}
