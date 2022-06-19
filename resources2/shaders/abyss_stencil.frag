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
} n;
layout(set = 0, binding = 3) uniform T {
    vec4 tf[29];
    // vec2 pos;
    // vec2 size;
    // vec4 data;
} t;

void main() {
    vec2 st = vec2(tex.x, 1.0 - tex.y);
    vec3 c = vec3(0.0);

    for (int i = 0; i < n.n; i++) {
        vec2 pos = t.tf[i].xy;
        vec2 size = t.tf[i].zw;

        float mask = step(pos.x, st.s) *
                     step(pos.y, st.t) *
                     (1.0 - step(pos.x + size.x, st.s)) *
                     (1.0 - step(pos.y + size.y, st.t));

        vec2 uv = (st - pos) / size;

        c += mask * texture(sampler2D(imgs[i], samp), uv).rgb;
    }

    color = vec4(c, 1.0);
}
