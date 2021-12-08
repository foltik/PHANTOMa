#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_nonuniform_qualifier : require

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform texture2D imgs[];
layout(set = 0, binding = 1) uniform sampler samp;
layout(set = 0, binding = 2) uniform N {
    uint n;
} n;
layout(set = 1, binding = 0) uniform U {
    float fr;
    float dx;
    float dy;
} u;

void main() {
    vec2 st = vec2(tex.x, 1.0 - tex.y);

    float fr = u.fr;
    float s;
    bool neg = false;
    if (u.dy != 0.0) {
        s = st.y;
        if (u.dy < 0) {
            neg = true;
            if (fr >= 0.0)
                fr = 2.0 - fr;
        }
    } else {
        s = st.x;
        if (u.dx < 0) {
            neg = true;
            if (fr >= 0.0)
                fr = 2.0 - fr;
        }
    }

    float i0 = 0.0;
    float m0 = 0.0;

    float i1 = 0.0;
    float m1 = 0.0;

    if (fr < 0) {
        if (neg)
            fr = 1.0 - (1.0 + fr);
        i0 = s - 1.0 + (1.0 - fr);
        m0 = step(fr, s) * (1.0 - step(fr + 1.0, s));
    } else if (fr < 1.0) {
        i0 = s + 1.0 - fr;
        i1 = s + 1.0 - (fr + 1.0);
        m0 = step(fr - 1.0, s) * (1.0 - step(fr, s));
        m1 = step(fr, s) * (1.0 - step(fr + 1.0, s));
    } else {
        i0 = s + 1.0 - fr;
        i1 = s + 1.0 - (fr - 1.0);
        m0 = step(fr - 1.0, s) * (1.0 - step(fr, s));
        m1 = step(fr - 2.0, s) * (1.0 - step(fr - 1.0, s));
    }

    vec3 c = vec3(0.0);
    if (u.dy != 0.0) {
        c += m0 * texture(sampler2D(imgs[0], samp), vec2(st.x, i0)).rgb;
        c += m1 * texture(sampler2D(imgs[1], samp), vec2(st.x, i1)).rgb;
    } else {
        c += m0 * texture(sampler2D(imgs[0], samp), vec2(i0, st.y)).rgb;
        c += m1 * texture(sampler2D(imgs[1], samp), vec2(i1, st.y)).rgb;
    }
    color = vec4(c, 1.0);
}
