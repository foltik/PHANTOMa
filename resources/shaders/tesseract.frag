#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_nonuniform_qualifier : require

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform U {
    vec3 c;
    float sx;
    float sy;
    float t;
} u;

// https://www.shadertoy.com/view/7dtXDs

#define PI 3.1415926536

// The MIT License
// Copyright © 2021 Inigo Quilez
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions: The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software. THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

// Distance to a quad.
//
// List of other 2D distances: https://www.shadertoy.com/playlist/MXdSRf
//
// and www.iquilezles.org/www/articles/distfunctions2d/distfunctions2d.htm
//
// Gradient of a quad here: https://www.shadertoy.com/view/WtVcD1


// signed distance to a 2D quad
float sdQuad( in vec2 p, in vec2 p0, in vec2 p1, in vec2 p2, in vec2 p3 )
{
	vec2 e0 = p1 - p0; vec2 v0 = p - p0;
	vec2 e1 = p2 - p1; vec2 v1 = p - p1;
	vec2 e2 = p3 - p2; vec2 v2 = p - p2;
	vec2 e3 = p0 - p3; vec2 v3 = p - p3;

	vec2 pq0 = v0 - e0*clamp( dot(v0,e0)/dot(e0,e0), 0.0, 1.0 );
	vec2 pq1 = v1 - e1*clamp( dot(v1,e1)/dot(e1,e1), 0.0, 1.0 );
	vec2 pq2 = v2 - e2*clamp( dot(v2,e2)/dot(e2,e2), 0.0, 1.0 );
    vec2 pq3 = v3 - e3*clamp( dot(v3,e3)/dot(e3,e3), 0.0, 1.0 );
    
    vec2 ds = min( min( vec2( dot( pq0, pq0 ), v0.x*e0.y-v0.y*e0.x ),
                        vec2( dot( pq1, pq1 ), v1.x*e1.y-v1.y*e1.x )),
                   min( vec2( dot( pq2, pq2 ), v2.x*e2.y-v2.y*e2.x ),
                        vec2( dot( pq3, pq3 ), v3.x*e3.y-v3.y*e3.x ) ));

    float d = sqrt(ds.x);

	return (ds.y>0.0) ? -d : d;
}


// The MIT License
// Copyright © 2014 Inigo Quilez
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions: The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software. THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

// Somehow optimized HSV and HSL to RGB conversion functions. 

//========================================================================

const float eps = 0.0000001;


vec3 hsv2rgb( in vec3 c )
{
    vec3 rgb = clamp( abs(mod(c.x*6.0+vec3(0.0,4.0,2.0),6.0)-3.0)-1.0, 0.0, 1.0 );
    return c.z * mix( vec3(1.0), rgb, c.y);
}

vec3 hsl2rgb( in vec3 c )
{
    vec3 rgb = clamp( abs(mod(c.x*6.0+vec3(0.0,4.0,2.0),6.0)-3.0)-1.0, 0.0, 1.0 );
    return c.z + c.y * (rgb-0.5)*(1.0-abs(2.0*c.z-1.0));
}

vec3 rgb2hsv( in vec3 c)
{
    vec4 k = vec4(0.0, -1.0/3.0, 2.0/3.0, -1.0);
    vec4 p = mix(vec4(c.zy, k.wz), vec4(c.yz, k.xy), (c.z<c.y) ? 1.0 : 0.0);
    vec4 q = mix(vec4(p.xyw, c.x), vec4(c.x, p.yzx), (p.x<c.x) ? 1.0 : 0.0);
    float d = q.x - min(q.w, q.y);
    return vec3(abs(q.z + (q.w - q.y) / (6.0*d+eps)), d / (q.x+eps), q.x);
}

vec3 rgb2hsl( vec3 col )
{
    float minc = min( col.r, min(col.g, col.b) );
    float maxc = max( col.r, max(col.g, col.b) );
    vec3  mask = step(col.grr,col.rgb) * step(col.bbg,col.rgb);
    vec3 h = mask * (vec3(0.0,2.0,4.0) + (col.gbr-col.brg)/(maxc-minc + eps)) / 6.0;
    return vec3( fract( 1.0 + h.x + h.y + h.z ),              // H
                 (maxc-minc)/(1.0-abs(minc+maxc-1.0) + eps),  // S
                 (minc+maxc)*0.5 );                           // L
}

mat4 rot(float wx, float wy, float wz, float xy, float xz, float yz) {
    mat4 result = mat4(cos(wx), 0, 0, -sin(wx),
                       0, 1, 0, 0,
                       0, 0, 1, 0,
                       sin(wx), 0, 0, cos(wx)
                       );
    result *= mat4(1, 0, 0, 0,
                   0, cos(wy), 0, -sin(wy),
                   0, 0, 1, 0,
                   0, sin(wy), 0, cos(wy)
                   );
    result *= mat4(1, 0, 0, 0,
                   0, 1, 0, 0,
                   0, 0, cos(wz), -sin(wz),
                   0, 0, sin(wz), cos(wz)
                   );
    result *= mat4(cos(xy), -sin(xy), 0, 0,
                   sin(xy), cos(xy), 0, 0,
                   0, 0, 1, 0,
                   0, 0, 0, 1
                   );
    result *= mat4(cos(xz), 0, -sin(xz), 0,
                   0, 1, 0, 0,
                   sin(xz), 0, cos(xz), 0,
                   0, 0, 0, 1
                   );
    result *= mat4(1, 0, 0, 0,
                   0, cos(yz), -sin(yz), 0,
                   0, sin(yz), cos(yz), 0,
                   0, 0, 0, 1
                   );
    return result;
}

float inQuad(vec2 p, vec2 a, vec2 b, vec2 c, vec2 d) {
    return smoothstep(2e-3, -2e-3, sdQuad(p, a, b, c, d)) +
           smoothstep(2e-3, -2e-3, sdQuad(p, d, c, b, a));
}

vec3 colCell(vec2 p) {
    return hsv2rgb(vec3(atan(p.y,p.x) / (2.*PI), 1, 0.3));
}

void main() {
    vec2 st = vec2(tex.x, 1.0 - tex.y);
    float tt = u.t * 1;

    vec2 xy = st * 2.0 - 1.0;
    xy /= vec2(u.sx, u.sy);
    vec2 m = vec2(0.0);

    float t = 0.5 * smoothstep(0., 1., fract(0.1 * tt));
    mat4 R = rot(1.75*PI + m.x, 1.5*PI + m.y, PI*(1.25 + 2.*t), PI*(0.5 + 3.*t), PI*(1.75 + 5.*t), PI*(1. + 7.*t));
    vec3 col = vec3(0);

    #define H vec3(0.5, -0.5, 0)
    #define FACE(i0,i1,i2,i3) mask += inQuad(xy, (R*H.i0).xy, (R*H.i1).xy, (R*H.i2).xy, (R*H.i3).xy);
    #define FACExx(idx) FACE(xxxx.idx, xxxy.idx, xxyy.idx, xxyx.idx)
    #define FACExy(idx) FACE(xyxx.idx, xyxy.idx, xyyy.idx, xyyx.idx)
    #define FACEyx(idx) FACE(yxxx.idx, yxxy.idx, yxyy.idx, yxyx.idx)
    #define FACEyy(idx) FACE(yyxx.idx, yyxy.idx, yyyy.idx, yyyx.idx)

    #define CELLx(idx) { \
        float mask = 0.; \
        FACExx(xyzw.idx); FACExx(xzyw.idx); FACExx(xzwy.idx); \
        FACExy(xyzw.idx); FACExy(xzyw.idx); FACExy(xzwy.idx); \
        col += clamp(mask, 0., 1.) * colCell((R*H.xzzz.idx).xy); \
    }
    #define CELLy(idx) { \
        float mask = 0.; \
        FACEyx(xyzw.idx); FACEyx(xzyw.idx); FACEyx(xzwy.idx); \
        FACEyy(xyzw.idx); FACEyy(xzyw.idx); FACEyy(xzwy.idx); \
        col += clamp(mask, 0., 1.) * colCell((R*H.yzzz.idx).xy); \
    }

    CELLx(xyzw);
    CELLy(xyzw);
    CELLx(yxzw);
    CELLy(yxzw);
    CELLx(yzxw);
    CELLy(yzxw);
    CELLx(yzwx);
    CELLy(yzwx);

    color = vec4(sqrt(col), 1);
}
