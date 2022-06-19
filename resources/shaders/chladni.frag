#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_nonuniform_qualifier : require

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform U {
    vec3 c;
    float t;
    float x;
    float y;
} u;

// https://www.shadertoy.com/view/4dXSD2

const float pi = 3.1415926535897932384;

// Edge lengths of the rectangular plate. Note that in reality only 
// for the case a=b degenerate eigenmodes appear, leading to the 
// superimposition as implemented here.
float a = 1.0;
float b = 1.0;

// Chladni eigenmodes
float chladni( float m, float n, vec2 uv )
{	
	// cos()*cos() for modes of a plate fixed at its center
	// sin()*sin() for modes of a plate fixed at its border (boring)
	return cos(n*pi*uv.x/a)*cos(m*pi*uv.y/b);
}

// Eigenfrequencies (not used)
float omega( float m, float n )
{
	const float rho = 1.0;
	const float eta = 1.0;	
	return pi * sqrt( (rho/eta) * (m/a)*(m/a) + (n/b)*(n/b) );
}

void main() {
    vec2 st = vec2(tex.x, tex.y);
    float t = u.t * 1;

	// Domain [-.5,-.5]x[.5,.5]
    vec2 uv = st - 0.5;
	
	// Knot numbers  
	vec2 mn = 10.0 * vec2(u.x, u.y);
    
	// Superposition coefficients
	float theta = -t*0.5;
	mat2 R = mat2( cos(theta), sin(theta), -sin(theta), cos(theta) );
	vec2 c = R * vec2(1.0,-1.0);
	
	// Superposition of eigenmodes
	float u = c.x*chladni(mn.x,mn.y,uv) + c.y*chladni(mn.y,mn.x,uv);
	
	// Shift-scale from [-1,1] to [0,1]		
	u = (0.5+u/2.0);
	
	// Visualize knot lines (i.e. zero-crossings)
    float lines = 0.0;
    float thickness = 0.015;
    for(float delta=-0.6; delta < 1.2; delta+=0.2)
        //u += step( abs(v-delta), 0.5*thickness );
        lines += smoothstep(abs(u-delta)-thickness, abs(u-delta)+thickness, thickness);

    // Cineshader uses alpha for 2.5D depth effect
    float alpha = 0.0;

    
	color = vec4(lines*vec3(1.0),alpha);
}
