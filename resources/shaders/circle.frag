#version 450
#
layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform U {
    float t;
} u;

void main() {
    vec2 uv = (tex * 2.0) - 1.0;

    float d = distance(uv, vec2(0.0));
    float dr = length(vec2(dFdx(uv.x), dFdy(uv.y)));

    float r = 0.25 + 0.1 * sin(u.t);
    float w = 0.03;

    float c = smoothstep(r - dr,     r + dr,     d) 
            - smoothstep(r - dr + w, r + dr + w, d);

    color = vec4(vec3(c), 1.0);
}
