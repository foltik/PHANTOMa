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
    float n1;
    float n2;
    float dz;
    float thickness;
    float falloff;
    uint n;
} u;

/////////////////////////////////////////////////////////////////////////////
// XBE
// Retro style terrain rendering
// 
// https://www.shadertoy.com/view/4dfSDj
//

const float PI = 3.141592654;

// Noise from IQ
vec2 hash( vec2 p )
{
	p = vec2( dot(p,vec2(127.1,311.7)),
			 dot(p,vec2(269.5,183.3)) );
	return -1.0 + 2.0*fract(sin(p)*43758.5453123);
}

float noise( in vec2 p )
{
	const float K1 = 0.366025404;
	const float K2 = 0.211324865;
	
	vec2 i = floor( p + (p.x+p.y)*K1 );
	
	vec2 a = p - i + (i.x+i.y)*K2;
	vec2 o = (a.x>a.y) ? vec2(1.0,0.0) : vec2(0.0,1.0);
	vec2 b = a - o + K2;
	vec2 c = a - 1.0 + 2.0*K2;
	
	vec3 h = max( 0.5-vec3(dot(a,a), dot(b,b), dot(c,c) ), 0.0 );
	
	vec3 n = h*h*h*h*vec3( dot(a,hash(i+0.0)), dot(b,hash(i+o)), dot(c,hash(i+1.0)));
	
	return dot( n, vec3(70.0) );
}

const mat2 m = mat2( 0.80,  0.60, -0.60,  0.80 );

float fbm4( in vec2 p )
{
    float f = 0.0;
    f += 0.5000*noise( p ); p = m*p*2.02;
    f += 0.2500*noise( p ); p = m*p*2.03;
    f += 0.1250*noise( p ); p = m*p*2.01;
    f += 0.0625*noise( p );
    return f;
}

float fbm6( in vec2 p )
{
    float f = 0.0;
    f += 0.5000*noise( p ); p = m*p*2.02;
    f += 0.2500*noise( p ); p = m*p*2.03;
    f += 0.1250*noise( p ); p = m*p*2.01;
    f += 0.0625*noise( p ); p = m*p*2.04;
    f += 0.031250*noise( p ); p = m*p*2.01;
    f += 0.015625*noise( p );
    return f;
}

mat4 CreatePerspectiveMatrix(in float fov, in float aspect, in float near, in float far)
{
    mat4 m = mat4(0.0);
    float angle = (fov / 180.0) * PI;
    float f = 1. / tan( angle * 0.5 );
    m[0][0] = f / aspect;
    m[1][1] = f;
    m[2][2] = (far + near) / (near - far);
    m[2][3] = -1.;
    m[3][2] = (2. * far*near) / (near - far);
    return m;
}

mat4 CamControl( vec3 eye, float pitch)
{
    float cosPitch = cos(pitch);
    float sinPitch = sin(pitch);
    vec3 xaxis = vec3( 1, 0, 0. );
    vec3 yaxis = vec3( 0., cosPitch, sinPitch );
    vec3 zaxis = vec3( 0., -sinPitch, cosPitch );
    // Create a 4x4 view matrix from the right, up, forward and eye position vectors
    mat4 viewMatrix = mat4(
        vec4(       xaxis.x,            yaxis.x,            zaxis.x,      0 ),
        vec4(       xaxis.y,            yaxis.y,            zaxis.y,      0 ),
        vec4(       xaxis.z,            yaxis.z,            zaxis.z,      0 ),
        vec4( -dot( xaxis, eye ), -dot( yaxis, eye ), -dot( zaxis, eye ), 1 )
    );
    return viewMatrix;
}


void main() {
    vec2 st = vec2(tex.x, tex.y);
    float t = u.t * 1;
    vec2 res = vec2(u.w, u.h);

    vec2 p = (vec2(st.x, 1.0 - st.y) * 2.0) - 1.0;
    p.x /= res.y / res.x;

	vec3 eye = vec3(0., /*0.5+0.25*cos(0.5*t)*/0.85+0.03*cos(0.5*t), -1);
    mat4 projmat = CreatePerspectiveMatrix(50., res.x/res.y, 0.1, 10.);
    mat4 viewmat = CamControl(eye, -5.*PI/180.);
    mat4 vpmat = viewmat*projmat;
    
	vec3 col = vec3(0.);
	vec3 acc = vec3(0.);
	float d;
	
    vec4 pos = vec4(0.);
	float lh = -res.y;
	float off = 0.1*t;
	float h = 0.;
	float z = 0.1;
	float zi = u.dz * 0.1;
	for (int i=0; i<u.n; ++i)
	{
        pos = vec4(p.x, u.n1*fbm4(u.n2*vec2(eye.x+p.x, z+off)), eye.z+z, 1.);
        h = (vpmat*pos).y - p.y;
		if (h>lh)
		{
			d = abs(h);
			col = vec3( d<0.005?smoothstep(1.,0.,d*500*(1.0 - u.thickness)):0. );
			col *= exp(-u.falloff*float(i));
            acc += col;
			lh = h;
		}
		z += zi;
	}
	col = sqrt(clamp(acc, 0., 1.));

    col += vec3(p.y * 0.1);
	color = vec4(u.c * col, 1.0);
}
