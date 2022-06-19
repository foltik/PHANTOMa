#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_nonuniform_qualifier : require

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform U {
    vec3 c;
    float t;
    float w;
    float h;
    float dx;
    float freq;
} u;

// https://www.shadertoy.com/view/sscSR4

void main() {
    vec2 st = vec2(tex.x, tex.y);
    float tt = u.t * 1;
    vec2 R = vec2(u.w, u.h);

    vec2 U = st * R;
    U.y = 1.0 - U.y;

    vec2 H = fract( 4e5 * sin( ceil(U = 5.*(U+U - R)/R.y).x * R ));
    
    float t = tt * ( .5 + H.y ),
          x = U.y += 3.*H.x;
          x = floor( x-t -   .6*cos(x)  / ( 1.- .6*sin(x) )  ) + t;
    color = 1. - R.yyyy/15.* abs( length( vec2( U.y - x - .6*cos(x) -.5+.3*sin(x) , 
                                            fract( U + 0.3*u.dx*sin(x/(1.0 - (u.freq - 0.05))) ) -.5 )
                                    ) - mod( 8e4 * sin( H+(x-t) * R ), .2).y 
                            );
}
