#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_nonuniform_qualifier : require

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform U {
    vec3 c;
    float t;
    float sx;
    float sy;
    float r;
} u;

// https://www.shadertoy.com/view/wdlGRM

void main() {
    vec2 st = vec2(tex.x, tex.y);
    float t = u.t * 1;

    vec2 uv = st - 0.5;
    uv *= vec2(u.sx, u.sy);
    // uv.x /= 9.0 / 16.0;
    uv *= mat2(.707, -.707, .707, .707);
    uv *= 15.;
    //uv.x /= u.aspect;
    
    vec2 gv = fract(uv)-.5; 
	vec2 id = floor(uv);
    
	float m = 0.;
    float p;
    for(float y=-1.; y<=1.; y++) {
    	for(float x=-1.; x<=1.; x++) {
            vec2 offs = vec2(x, y);
            
            p = -t+length(id-offs)*.2;
            float r = mix(.4, 1.5, sin(p)*.5+.5) * u.r;
    		float c = smoothstep(r, r*.9, length(gv+offs));
    		m = m*(1.-c) + c*(1.-m);
        }
    }

    color = vec4(u.c * vec3(m), 1.0);
}
