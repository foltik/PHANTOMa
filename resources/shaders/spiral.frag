#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_nonuniform_qualifier : require

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform U {
    vec3 col;
    float aspect;
    float t;
    float swirl;
    float speed;
    float cutoff;
    float amount;
    uint spokes;
} u;

float aa_step(float thres, float x) {
    float dx = length(vec2(dFdx(x), dFdy(x)));
    return smoothstep(thres-dx, thres+dx, x);
}

void main() {
    // vec2 uv = tex * 2.0 - 1.0;
    // color = vec4(uv, 0.0, 1.0);

    vec2 uv = tex * 2.0 - 1.0;
    vec2 st = vec2(length(uv), atan(uv.y, uv.x));

    float v = aa_step(0, sin(4*u.swirl/st.x + u.spokes*st.y + 8*u.speed*u.t))
        * smoothstep(1-u.cutoff, 1-u.cutoff + 0.7, st.x);

    color = vec4(u.col, 1.0) * vec4(vec3(v), 1.0) * u.amount;
}
