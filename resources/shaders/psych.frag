#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform U {
    float t;
} u;

void main() {
    float f = fract(u.t);
    
    // set position
    vec2 v = vec2(1920.0, 1080.0);
    vec2 p = gl_FragCoord.xy;
    p = (p-v*.5)*.4 / v.y;
    // breathing effect
    p += p * sin(dot(p, p)*20.-u.t) * .04;
    
    // accumulate color
    vec4 c = vec4(0.0);
    for (int ii = 0; ii <= 7; ii++) {
        float i = 0.5 + float(ii);
        // fractal formula and rotation
        p = abs(2.*fract(p-.5)-1.) * mat2(cos(.01*(u.t)*i*i + .78*vec4(1,7,3,1)));
        
        // coloration
        c += exp(-abs(p.y)*5.) * (cos(vec4(2,3,1,0)*i)*.5+.5);
    }
    
    // palette
    c.gb *= .5;
    
    color = c;
}
