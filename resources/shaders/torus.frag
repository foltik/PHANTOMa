#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_nonuniform_qualifier : require

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform U {
    vec3 c;
    float w;
    float h;
    float t;
    float fov;
    float r;
    float glow;
    float thickness;
} u;

// https://www.shadertoy.com/view/fscGDH

void main() {
    vec2 st = vec2(tex.x, tex.y);
    st.y = 1.0 - st.y;

    vec3 R = vec3(u.w, u.h, 1.0);
    R.y *= 3.0 * (1.0 - u.fov);
    vec2 U = st * R.xy;

    float tt = u.t * 1;

    vec3  A = vec3(0,11,33),
          D = normalize(vec3( U+U, -R.y ) - R ), // ray direction
          p = 20./R;                             // marching point along ray 
    float l = 1.1, t=l;

    vec4 O = vec4(0.0);
     
    for ( ; l > 0. && t > .01 ; l-=.01 ) //  O.x>0. just for security
        O.xyz = p, 
        O.yz *= mat2( cos( A.xyzx + .5 + cos( tt + A ).y/4.)),                                     // rotations       
        O.w = length(O.xz *= mat2( cos( A.xyzx + .5 + cos( tt + A ).x/2.)) ) - (7.5 + 15.0*u.r), // R0=20   R1=17
        t = 17.0 - length(O.wy),                               // abs for inside + outside
        p += t*D;                                             // step forward = dist to obj
                                                              // texture                        
    U = min( abs(U = sin(18.*atan(O.xy,O.zw)) ) / fwidth(U) / (5.0 * u.thickness), 1.45 - (u.glow / 2.0)); // mesh
    color = vec4(u.c * vec3(1.0 - (l * U.x * U.y)), 1.0);
}
