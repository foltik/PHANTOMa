#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform texture2D imgs[];
layout(set = 0, binding = 1) uniform sampler samp;

layout(set = 0, binding = 0) uniform U {
    vec3 c;
    float t;
    float mx;
    float my;
    float amt;
} u;

vec2 grad(ivec2 z) {
    // 2D to 1D  (feel free to replace by some other)
    int n = z.x+z.y*11111;

    // Hugo Elias hash (feel free to replace by another one)
    n = (n<<13)^n;
    n = (n*(n*n*15731+789221)+1376312589)>>16;

    // Perlin style vectors
    n &= 7;
    vec2 gr = vec2(n&1,n>>1)*2.0-1.0;
    return ( n>=6 ) ? vec2(0.0,gr.x) : 
           ( n>=4 ) ? vec2(gr.x,0.0) :
                              gr;
}

float gradnoise(vec2 p) {
    ivec2 i = ivec2(floor( p ));
     vec2 f =       fract( p );
	
	vec2 u = f*f*(3.0-2.0*f); // feel free to replace by a quintic smoothstep instead

    return 0.5 + 0.5 * 
        mix( mix( dot( grad( i+ivec2(0,0) ), f-vec2(0.0,0.0) ), 
            dot( grad( i+ivec2(1,0) ), f-vec2(1.0,0.0) ), u.x),
        mix( dot( grad( i+ivec2(0,1) ), f-vec2(0.0,1.0) ), 
            dot( grad( i+ivec2(1,1) ), f-vec2(1.0,1.0) ), u.x), u.y);
}

#define tau 6.2831853

mat2 makem2(in float theta){float c = cos(theta);float s = sin(theta);return mat2(c,-s,s,c);}
// float noise( in vec2 x ){return texture(iChannel0, x*.01).x;}
float rand(vec2 p) {
    float dt = dot(p, vec2(12.9898, 78.233));
    float sn = mod(dt, 3.14);
    return fract(sin(sn) * 43758.5453);
}

float fbm(in vec2 p) {	
	float z=2.;
	float rz = 0.;
	vec2 bp = p;
	for (float i= 1.;i < 6.;i++)
	{
		rz+= abs((gradnoise(p * 10.0)-0.5)*1.0)/z;
		z = z*2.;
		p = p*2.;
	}
	return rz;
}

float dualfbm(float t, vec2 p) {
    //get two rotated fbm calls and displace the domain
	vec2 p2 = p*.7;
	vec2 basis = vec2(fbm(p2-t*1.6),fbm(p2+t*1.7));
	basis = (basis-.5)*.2;
	p += basis;
	
	//coloring
	return fbm(p*makem2(t*0.2));
}

void main() {
    vec2 st = vec2(tex.x, 1.0 - tex.y);
    float t = u.t * 0.1;

    vec2 p = st * 2.0 - 1.0;
    p *= vec2(u.mx, u.my);

    // vec3(0.2, 0.1, 0.4)
	vec3 col = u.c / (dualfbm(t, p) * 250.0 * ((1.0 - u.amt) + 0.01));
	color = vec4(pow(col, vec3(0.99)), 1.0);
}
