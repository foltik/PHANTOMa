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
} u;

// https://www.shadertoy.com/view/3ttSR7

const float PI = 3.14159265358979323846264;
const int MAX_PRIMARY_RAY_STEPS = 80; // decrease this number if it runs slow on your computer


float sdTorus( vec3 p, vec2 t ) {
  vec2 q = vec2(length(p.xz)-t.x,p.y);
  return length(q)-t.y;
}

float distanceField(vec3 p) {
	return -sdTorus(p.yxz, vec2(5.0, 1.0));
}

vec3 castRay(vec3 pos, vec3 dir) {
	for (int i = 0; i < MAX_PRIMARY_RAY_STEPS; i++) {
			float dist = distanceField(pos);
			pos += dist * dir;
	}
	return pos;
}

float random (in float x) {
    return fract(sin(x)*1e4);
}

float random (in vec2 st) {
    return fract(sin(dot(st.xy, vec2(12.9898,78.233)))* 43758.5453123);
}

float pattern(vec2 st, vec2 v, float t) {
    vec2 p = floor(st+v);
    return step(t, random(25.+p*.000004)+random(p.x)*0.75 );
}

void main() {
    vec2 st = tex;
    float t = u.t * 1;

    vec2 res = vec2(u.w, u.h);
    vec2 xy = st * res;

    for(float di=-0.25;di<=0.25;di+=.5){
        for(float dj=-0.25;dj<=0.25;dj+=.5){
            vec2 screenPos = ((xy + vec2(di,dj)) / res) * 2.0 - 1.0;
            vec3 cameraPos = vec3(0.0, 4.2, -3.8);

            // vec3 cameraDir = u.dir;//vec3(0., 0.22, 0.8);
            vec3 cameraDir = vec3(0., 0.12, 0.4);
            vec3 planeU = vec3(1.0, 0.0, 0.0) * 0.8;
            vec3 planeV = vec3(0.0, res.y / res.x * 1.0, 0.0);
            vec3 rayDir = normalize(cameraDir + screenPos.x * planeU + screenPos.y * planeV);

            vec3 rayPos = castRay(cameraPos, rayDir);

            float majorAngle = atan(rayPos.z, rayPos.y);
            float minorAngle = atan(rayPos.x, length(rayPos.yz) - 5.0);

            vec2 st = vec2(majorAngle/PI/2.0,minorAngle/PI);

            vec2 grid = vec2(1000.0,50.);
            st *= grid;

            vec2 ipos = floor(st);  // integer
            vec2 fpos = fract(st);  // fraction

            vec2 vel = vec2(t*0.09*max(grid.x,grid.y)); // time
            vel *= vec2(1.,0.0) *(0.4+2.0*pow(random(1.0+ipos.y),2.0)); // direction

            // For colorful stuff
            vec2 offset = 0.*vec2(0.2,0.25);

            vec3 col = vec3(0.);
            float replaceMouse = 0.75+0.45*sin(0.6*t + 0.015*st.x);
            col.r = pattern(st+offset,vel,replaceMouse);
            col.g = pattern(st,vel,replaceMouse);
            col.b = pattern(st-offset,vel,replaceMouse);
            col *= u.c;

            // Margins
            col *= step(0.2,fpos.y);

            color += 0.25*vec4(col,1.0);
        }
    }
}
