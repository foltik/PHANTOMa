#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform texture2D imgs[];
layout(set = 0, binding = 1) uniform sampler samp;

layout(set = 1, binding = 0) uniform Uniforms {
    float t;
} u;

float inside(float x, float lo, float hi) {
    return step(lo, x) - step(hi, x);
}

float rand(vec2 p) {
    float dt = dot(p, vec2(12.9898, 78.233));
    float sn = mod(dt, 3.14);
    return fract(sin(sn) * 43758.5453);
}

float rrand(vec2 p, float lo, float hi) {
    return lo + rand(p) * (hi - lo);
}

void main() {
    vec2 st = vec2(tex.x, tex.y);
    vec3 c = texture(sampler2D(imgs[0], samp), st).rgb;

    float t = floor(u.t * 10000.0 * 50.0);
    float r = rand(vec2(t, 0.0));

    float amt = 0.5;
    const float f_skew = 0.08 * amt;
    const float f_color = 0.001 * amt;

    // Skew X
    for (float i = 0.0; i < 20.0 * f_skew; i += 1.0) {
        float y = rand(vec2(t, i));
        float h = rand(vec2(t, i + 1.0)) * 0.25;

        if (inside(st.y, y, fract(y + h)) == 1.0) {
            float ofs = rrand(vec2(t, i + 2.0), -f_skew, f_skew);
            c = texture(sampler2D(imgs[0], samp), vec2(st.x + ofs, st.y)).rgb;
        }
    }

    // Channel shift
    float cx = rrand(vec2(t, 1.0), -f_color, f_color);
    float cy = rrand(vec2(t, 2.0), -f_color, f_color);
    vec2 cofs = vec2(cx, cy);
    if (r <= 0.33) {
        c.r = texture(sampler2D(imgs[0], samp), st + cofs).r;
    } else if (r <= 0.66) {
        c.g = texture(sampler2D(imgs[0], samp), st + cofs).g;
    } else {
        c.b = texture(sampler2D(imgs[0], samp), st + cofs).b;
    }

    color = vec4(c, 1.0);
}
