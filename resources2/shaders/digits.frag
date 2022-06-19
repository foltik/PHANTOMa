#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_nonuniform_qualifier : require

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform U {
    vec3 c;
    float t;
    int i;
} u;

// https://www.shadertoy.com/view/NscGz2

const float[10] DIGIT = float[10](126.0, 48.0, 109.0, 121.0, 51.0, 91.0, 95.0, 114.0, 127.0, 123.0);

float random (vec2 st) {
    return fract(sin(41.3455422+dot(st.xy,
                         vec2(12.9898,78.233)))*
        43758.5453123);
}

vec2 random2(vec2 st) {
    return vec2(
        random(st),
        random(
            vec2(
                random(st),
                random(st + vec2(24.2145, 346.234))
            )
        )
    );
}

const float blockSize = 3.0;

float block(vec2 pos, vec4 set, float flag) {
    vec2 temp = abs(pos - set.xy) - set.zw * 0.5;
    return mix(1.0, step(0.0, max(temp.x, temp.y)), flag);
}
float map(vec2 pos, vec2 index, float flag, float t) {
    vec2 limPos = floor(pos * vec2(5.0, 8.0)) / vec2(5.0, 8.0);
    float limTime = floor(t * 10.0) / 10.0;
    float pulse = step(0.98, random(limPos + index + vec2(limTime)));
    vec2 pulseDelta = (random2(limPos + index + vec2(limTime)) * 2.0 - vec2(1.0)) * 0.07 * pulse;
    vec2 delta = (random2(index + vec2(limTime)) * 2.0 - vec2(1.0)) * 0.01;
    
    return min(
        block(pos + delta + pulseDelta, vec4(0.5, 0.8, 0.8, 0.2), mod(floor(flag / 64.0), 2.0)),
        min(
            block(pos + delta + pulseDelta, vec4(0.8, 0.65, 0.2, 0.5), mod(floor(flag / 32.0), 2.0)),
        min(
            block(pos + delta + pulseDelta, vec4(0.8, 0.35, 0.2, 0.5), mod(floor(flag / 16.0), 2.0)),
        min(
            block(pos + delta + pulseDelta, vec4(0.5, 0.2, 0.8, 0.2), mod(floor(flag / 8.0), 2.0)),
        min(
            block(pos + delta + pulseDelta, vec4(0.2, 0.35, 0.2, 0.5), mod(floor(flag / 4.0), 2.0)),
        min(
            block(pos + delta + pulseDelta, vec4(0.2, 0.65, 0.2, 0.5), mod(floor(flag / 2.0), 2.0)),
            block(pos + delta + pulseDelta, vec4(0.5, 0.5, 0.8, 0.2), mod(flag, 2.0))
        )))))
    );
}

float getPattern(vec2 uv, float t, float limTime) {
    vec2 index = vec2(0.0);
    vec2 pUv = uv;
    
    for(float i = 1.0; i < pow(blockSize, 8.0);i *= blockSize) {
        index += floor(pUv * blockSize) / blockSize / i;
        
        vec2 limUv = floor(pUv * blockSize);
        
        pUv = mod(uv, vec2(1.0 / blockSize / i)) * blockSize * i;
        
        float v = random(limUv + vec2(limTime));
        if(0.7 > v) break;
        
    }
    
    return map(pUv, index, DIGIT[int(floor(random(index) * 10.0 + t * 10.0)) % 10], t);
}

const float interval = 1.0;

float easeInOutExpo(float x) {
    return x == 0.0
      ? 0.0
      : x == 1.0
      ? 1.0
      : x < 0.5 ? pow(2.0, 20.0 * x - 10.0) / 2.0
      : (2.0 - pow(2.0, -20.0 * x + 10.0)) / 2.0;
}

void main() {
    vec2 uv = vec2(tex.x, 1.0 - tex.y);
    uv.y = 1.0 - uv.y;

    vec3 v = vec3(1.0 - getPattern(uv, u.t, u.i));
    color = vec4(u.c * v, 1.0);
}
