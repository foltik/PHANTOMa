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
    float scale;
    int dir;
    int rev;
} u;

void main() {
    vec2 st = vec2(tex.x, 1.0 - tex.y);

    // float s;
    // bool rev = false;
    // if (dx != 0.0) {
    //     s = st.x;
    //     rev = dx < 0.0;
    // } else {
    //     s = st.y;
    //     rev = dy < 0.0;
    // }

    float s;
    if (u.dir == 0) {
        s = st.y;
    } else {
        s = st.x;
    }

    float fr;
    if (u.rev > 0) {
        fr = u.fr;
    } else {
        fr = 1.0 - u.fr;
    }

    float v = (s / u.scale) - fr;

    vec2 uv;
    if (u.dir == 0) {
        uv = vec2(st.x, v);
    } else {
        uv = vec2(v, st.y);
    }

    vec3 c = texture(sampler2D(imgs[0], samp), uv).rgb;

    float mask = 1.0;
    if (u.fr < 0.0) {
        if (u.rev > 0)
            mask = 1.0 - step(1.0 + u.fr, (s / u.scale));
        else
            mask = step(1.0 - (1.0 + u.fr), (s / u.scale));
    }

    color = vec4(mask * c, 1.0);
}
