#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform U {
    vec3 col;
    int n;
    int dir;
    float duty;
    float dl;
    float dr;
} u;

void main() {
    vec2 st = vec2(tex.x, 1.0 - tex.y);

    float s = 0.0;
    if (u.dir != 0)
        s = st.x;
    else
        s = st.y;

    float nn = (1.0 / u.n);
    float duty = (1.0 - u.duty) / u.n;

    float v = mod(s, nn);
    v = smoothstep(duty, duty + (nn * (u.dl / 2.0)), v) *
        smoothstep(0, nn * (u.dr / 2.0), nn - v);

    color = vec4(u.col.rgb * vec3(v), 1.0);
}
