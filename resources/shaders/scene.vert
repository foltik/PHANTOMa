#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 tex;
layout(location = 2) in vec3 norm;

layout(location = 0) out vec2 otex;
layout(location = 1) out vec3 onorm;
layout(location = 2) out vec3 opos;

layout(set = 1, binding = 0) uniform Effects {
    vec4 vals;
} fx;
layout(set = 1, binding = 1) uniform Camera {
    mat4 view;
    mat4 proj;
} cam;

layout(set = 2, binding = 0) uniform Model {
    mat4 transform;
} m;

layout(set = 3, binding = 0) uniform Object {
    mat4 transform;
} o;

float rand(vec2 p) {
    float dt = dot(p, vec2(12.9898, 78.233));
    float sn = mod(dt, 3.14);
    return fract(sin(sn) * 43758.5453);
}

void main() {
    float t = fx.vals.x;
    float glitch = fx.vals.y;
    float liquefy = fx.vals.z;

    vec3 gpos = pos;
    float ox = rand(pos.xy * pos.z * t) * 2.0 - 1.0;
    float oy = rand(pos.xy * pos.y * pos.z * t + 1242.0) * 2.0 - 1.0;
    float oz = rand(pos.xy * pos.y * pos.z * t + 9383.0) * 2.0 - 1.0;
    gpos += (rand(pos.xy + 2.0 * t) * 2.0 - 1.0) * glitch * 0.5;
    gpos = vec3(ox * glitch * 0.2 + gpos.x, oy * glitch * 0.2 + gpos.y, oz * glitch * 0.2 + gpos.z); 

    vec3 xpos = (o.transform * m.transform * vec4(gpos, 1.0)).xyz;

    float d2 = xpos.x * xpos.x + xpos.z * xpos.z;
    xpos.y += 5.0 * sin(d2 * sin(t / 143.0) / 8000.0) * liquefy;
    float y = xpos.y;
    float x = xpos.x;
    float om = sin(d2 * sin(t / 256.0) / 2000.0) * sin(t / 400.0);
    xpos.y = mix(y, x * sin(om) + y * cos(om), liquefy);
    xpos.x = mix(x, x * cos(om) - y * sin(om), liquefy);

    otex = tex;
    onorm = (o.transform * m.transform * vec4(norm, 1.0)).xyz;
    //opos = (o.transform * m.transform * vec4(xpos, 1.0)).xyz;
    opos = xpos;
    //gl_Position = cam.proj * cam.view * o.transform * m.transform * vec4(xpos, 1.0);
    gl_Position = cam.proj * cam.view * vec4(xpos, 1.0);
}
