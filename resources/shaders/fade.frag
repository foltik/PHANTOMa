#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform texture2D img;
layout(set = 0, binding = 1) uniform sampler samp;
layout(set = 0, binding = 2) uniform Uniforms {
    float t;
    float tc;
    float pause;
    float glitch;
    float glitch_mo;
    float vhs;
    float red;
    float flash;
    float shake;
    float black;
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

vec3 permute(vec3 x) { return mod(((x*34.0)+1.0)*x, 289.0); }
float snoise(vec2 p) {
    const vec4 C = vec4(0.211324865405187, 0.366025403784439, -0.577350269189626, 0.024390243902439);
    vec2 i = floor(p + dot(p, C.yy));
    vec2 x0 = p - i + dot(i, C.xx);
    vec2 i1 = (x0.x > x0.y) ? vec2(1.0, 0.0) : vec2(0.0, 1.0);
    vec4 x12 = x0.xyxy + C.xxzz;
    x12.xy -= i1;
    i = mod(i, 289.0);
    vec3 pm = permute(permute(i.y + vec3(0.0, i1.y, 1.0)) + i.x + vec3(0.0, i1.x, 1.0 ));
    vec3 m = max(0.5 - vec3(dot(x0, x0), dot(x12.xy, x12.xy), dot(x12.zw, x12.zw)), 0.0);
    m = m*m;
    m = m*m;
    vec3 x = 2.0 * fract(pm * C.www) - 1.0;
    vec3 h = abs(x) - 0.5;
    vec3 ox = floor(x + 0.5);
    vec3 a0 = x - ox;
    m *= 1.79284291400159 - 0.85373472095314 * (a0 * a0 + h * h);
    vec3 g;
    g.x  = a0.x  * x0.x  + h.x  * x0.y;
    g.yz = a0.yz * x12.xz + h.yz * x12.yw;
    return 130.0 * dot(m, g);
}

vec4 shake(float t, float amt) {
    vec2 uv = tex;

    uv.s += (rand(vec2(t)) - 0.5) * 0.050 * amt;
    uv.t += (rand(vec2(t + 100.0)) - 0.5) * 0.050 * amt;

    return texture(sampler2D(img, samp), uv);
}

vec4 cloth(float t, float amt) {
    vec2 uv = tex;

    uv.s += (rand(vec2(t, tex.x)) - 0.5) * 0.030 * amt;
    uv.t += (rand(vec2(t)) - 0.5) * 0.030 * amt;

    float ox = (rand(vec2(t + 00.0)) - 0.5) * 0.10 * amt;
    float oy = (rand(vec2(t + 10.0)) - 0.5) * 0.10 * amt;
    uv += vec2(ox, oy);

    return texture(sampler2D(img, samp), uv);
}

vec4 pause(float t, float amt) {
    vec2 uv = tex;
    vec4 c = vec4(0.0);

    uv.x += (rand(vec2(t, tex.y)) - 0.5) * 0.015 * amt;
    uv.y += (rand(vec2(t)) - 0.5) * 0.030 * amt;
    vec4 img = texture(sampler2D(img, samp), uv);

    float cf1 = rand(vec2(t, tex.y + 0.0));
    float cf2 = rand(vec2(t, tex.y + 1.0));
    float cf3 = rand(vec2(t, tex.y + 2.0));
    vec4 band = img * vec4(cf1, cf2, cf3, 0.0) * 0.8 * amt;

    float noise = rand(vec2(floor(uv.y * 80.0), floor(uv.x * 50.0)) + vec2(t, 0.0));
    if (noise <= -18.5 + 30.0 * uv.y * amt && noise >= -3.5 + 5.0 * uv.y)
        c = vec4(0.8) + band;
    else
        c = img + band;

    return c;
}

vec3 glitch_blocks(float tt, float amt) {
    vec3 c = texture(sampler2D(img, samp), tex).rgb;

    float t = floor(tt * 10000.0 * 50.0);
    float r = rand(vec2(t, 0.0));

    const float f_skew = 0.08 * amt;
    const float f_color = 0.001 * amt;

    // Skew X
    for (float i = 0.0; i < 20.0 * f_skew; i += 1.0) {
        float y = rand(vec2(t, i));
        float h = rand(vec2(t, i + 1.0)) * 0.25;

        if (inside(tex.y, y, fract(y + h)) == 1.0) {
            float ofs = rrand(vec2(t, i + 2.0), -f_skew, f_skew);
            c = texture(sampler2D(img, samp), vec2(tex.x + ofs, tex.y)).rgb;
        }
    }

    // Channel shift
    float cx = rrand(vec2(t, 1.0), -f_color, f_color);
    float cy = rrand(vec2(t, 2.0), -f_color, f_color);
    vec2 cofs = vec2(cx, cy);
    if (r <= 0.33) {
        c.r = texture(sampler2D(img, samp), tex + cofs).r;
    } else if (r <= 0.66) {
        c.g = texture(sampler2D(img, samp), tex + cofs).g;
    } else {
        c.b = texture(sampler2D(img, samp), tex + cofs).b;
    }

    return c;
}

vec3 glitch_vhs(float tt, float amt) {
    float t = tt * 100.0;

    const float f_a_wav = 0.42857 * 2 * amt;
    const float f_b_wav = 0.15011 * 2 * amt;

    // Layered noise
    float a_wav = max(0.0, (snoise(vec2(t,        tex.y * 0.3)) - 0.3)) * f_a_wav;
    float b_wav =          (snoise(vec2(t * 10.0, tex.y * 2.4)) - 0.5)  * f_b_wav;
    float n = a_wav + b_wav;

    // Skew X
    float x = tex.x - n * n * 0.25;
    vec3 c = texture(sampler2D(img, samp), vec2(x, tex.y)).rgb;

    // Interference lines
    c.rgb = mix(c.rgb, vec3(rand(vec2(tex.y * t))), n * 0.02).rgb;

    // Dark lines
    if (floor(mod(tex.y * 0.25, 2.0)) == 0.0)
        c.rgb *= 1.0 - (0.15 * n);

    // Channel shift and dim
    c.g = mix(c.r, texture(sampler2D(img, samp), vec2(x + n * 0.05, tex.y)).g, 1.0 - (0.5 * amt));
    c.b = mix(c.r, texture(sampler2D(img, samp), vec2(x - n * 0.05, tex.y)).b, 1.0 - (0.5 * amt));

    return c;
}

void main() {
    vec3 img = texture(sampler2D(img, samp), tex).rgb;
    color = vec4(img, 1.0 - u.black);
}
