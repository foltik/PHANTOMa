#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_nonuniform_qualifier : require

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform U {
    vec3 c;
    float t;
    float nx;
    float ny;
    float dx;
    float dy;
    float twin;
    int op;
} u;

float is_prime(int num) {
     if (num <= 1) return 0.0;
     if (num % 2 == 0 && num > 2) return 0.0;

     for(int i = 3; i < int(floor(sqrt(float(num)))); i+= 2)
     {
         if (num % i == 0)
             return 0.0;
     }
     return 1.0;
}

float is_twin_prime(int num) {
    if(is_prime(num) > 0.5)
        return float((is_prime(num + 2) > 0.5) || (is_prime(num - 2) > 0.5));
    return 0.0;
}

void main() {
    vec2 st = vec2(tex.x, tex.y);
    float t = u.t;

    vec2 uv = (1.0 - st) * vec2(u.nx, u.ny);
    int a = int(uv.x + t * u.dx);
    int b = int(uv.y + t * u.dy);

    int v;
    if      (u.op == 0) { v = a ^ b; }
    else if (u.op == 1) { v = a & b; }
    else if (u.op == 2) { v = a | b; }

    float p = mix(is_prime(v), is_twin_prime(v), u.twin);

    color = vec4(u.c * vec3(p), 1.0);
}
