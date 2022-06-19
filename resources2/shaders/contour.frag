#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_nonuniform_qualifier : require

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform U {
    vec3 c;
    float t;
    float min;
    float amp;
} u;

// https://www.shadertoy.com/view/lltBWM

float wave(float x, float y) {
    return sin(10.0*x+10.0*y) / 5.0 +
           sin(20.0*x+15.0*y) / 3.0 +
           sin(4.0*x+10.0*y) / -4.0 +
           sin(y) / 2.0 +
           sin(x*x*y*20.0) + 
           sin(x * 20.0 + 4.0) / 5.0 +
           sin(y * 30.0) / 5.0 + 
    	   sin(x) / 4.0; 
}

void main() {
    vec2 st = vec2(tex.x, 1.0 - tex.y);
    float t = u.t * 1;

    float z = wave(st.x, st.y) + 2.0;
    
    z *= u.amp * sin(1.57 + t / 5.0) + u.min;
    float d = fract(z);
    if(mod(z, 2.0) > 1.) d = 1.-d;
     
    d = d/fwidth(z);

    color = vec4(1.0 - d);
}