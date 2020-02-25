#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(binding = 0, std140) uniform Uniforms {
    float t;
    float aspect;
} u;

float mandelbrot(vec2 c) {
    const bool smoo = false;

    float c2 = dot(c, c);
    if ((256.0 * c2 * c2) - (96.0 * c2) + (32.0 * c.x) - 3.0 < 0.0) return 0.0;
    if (16.0 * (c2 + (2.0 * c.x) + 1.0) - 1.0 < 0.0) return 0.0;

    float l = 0.0;
    vec2 z  = vec2(0.0);

    const int iter = 512;

    int i = 0;
    for(int it = 0; it < iter; it++) {
        z = vec2(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y) + c;
        if (length(z) > 2.0) break;
        i++;
    }
    if (i == iter) return 0.0;

    return (float(i) - (!smoo ? 0.0 : log2(log2(dot(z, z))) - 4.0)) / float(iter);
}

void main() {
    vec2 p = (tex * 2.0) - 1.0;
    p.x *= u.aspect;

    float time = u.t / 1000.0;

    float zoom = 0.375 * sin(time * 0.1) + 0.625;
    float zcos = cos(0.15 * (1.0 - zoom) * 1.0 * time);
    float zsin = sin(0.15 * (1.0 - zoom) * 1.0 * time);

    zoom = pow(zoom, 8.0);
    vec2 pos = vec2((p.x * zcos) - (p.y * zsin), (p.x * zsin) + (p.y * zcos));

    vec2 target = vec2(-0.730016, 0.29002);

    float fr = mandelbrot(target + pos * zoom);

    vec3 col = vec3(0.5 + 0.5 * cos(2.5 + fr * 75. + vec3(1.000,0.295,0.152)));

    color = vec4(col, 1.0);
}

/*
#ifdef GL_ES
precision mediump float;
#endif

uniform vec2 u_resolution;
uniform vec2 u_mouse;
uniform float u_time;

float mandelbrot(vec2 c) {
    const bool smooth = false;

    float c2 = dot(c, c);
    if ((256.0 * c2 * c2) - (96.0 * c2) + (32.0 * c.x) - 3.0 < 0.0) return 0.0;
    if (16.0 * (c2 + (2.0 * c.x) + 1.0) - 1.0 < 0.0) return 0.0;

    float l = 0.0;
    vec2 z  = vec2(0.0);

    const int iter = 512;

	int i;
    for(int it = 0; it < iter; it++) {
        z = vec2(z.x * z.x - z.y * z.y, 2.0 * z.x * z.y) + c;
        if (length(z) > 2.0) break;
        i++;
    }
    if (i == iter) return 0.0;

    return (float(i) - (!smooth ? 0.0 : log2(log2(dot(z, z))) - 4.0)) / float(iter);
}

void main() {
    vec2 st = vec2(gl_FragCoord.x, gl_FragCoord.y) / vec2(u_resolution.x, u_resolution.y);
    //st = vec2(st.x, 1.0 - st.y);

    float px = 1.0;
    float py = 1.0;
    st = (st * 2.0) - vec2(px, py);

    vec2 p = st;

    float time = u_time;

    float zoom = 0.375 * sin(time * 0.1) + 0.625;
    float zcos = cos(0.15 * (1.0 - zoom) * 0.005 * time);
    float zsin = sin(0.15 * (1.0 - zoom) * 0.005 * time);

    zoom = pow(zoom,8.0);
    vec2 pos = vec2((p.x * zcos) - (p.y * zsin), (p.x * zsin) + (p.y * zcos));

    vec2 target = vec2(-0.730016, 0.29002);

    float fr = mandelbrot(target + pos * zoom);

    vec3 col = vec3(0.5 + 0.5 * cos(2.5 + fr * 75. + vec3(1.000,0.395,0.152)));
    gl_FragColor = vec4(col, 1.0);
}
*/