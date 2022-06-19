#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_nonuniform_qualifier : require

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform U {
    vec3 c;
    float t;
    float speed;
    float mx;
    float my;
    float warp;
} u;

// https://www.shadertoy.com/view/WtdcWf

float hash11(float n) {
    return fract(sin(n) * 43758.5453123);
}

float sdTorus (in vec3 pos, in vec2 t , out vec2 uv) {
    vec2 q = vec2(length(pos.xz) - t.x, pos.y);
    uv = vec2(atan(pos.x,pos.z), atan(q.x,q.y));
    return length(q) - t.y;
}

float map (in vec3 pos, out vec2 uv) {
   float d = sdTorus(pos, vec2(1.0, u.warp), uv);
   return d;
}

void main() {
    vec2 st = vec2(tex.x, 1.0 - tex.y);
    float tt = u.t * u.speed * 2.0;

    vec2 p = (2.0 * vec2(st.x, 1.0 - st.y) - vec2(1.0));
    p *= vec2(u.mx, u.my);

    vec3 col = vec3(0.0);
    
    float an = 1.0*tt;
    
    vec3 ta = .5*vec3(sin(an), 0, -cos(an));
    vec3 ro = vec3(0, sin(an), 0);
    vec3 ww = normalize(ta - ro);
    vec3 uu = normalize(cross(ww,vec3(0,1,0)));
    vec3 vv = normalize(cross(uu,ww));
    vec3 rd = normalize(p.x*uu + p.y*vv + .4*ww);
    
    float t = 0.0;
    for(int i=0; i<200; i++)
    {
        vec3 pos = ro + t*rd;
        
        vec2 _;
        float h = map( pos , _);
        
        if (h<.0005)
            break;
            
        t += h;
    }
    
    if( t < 20.0 )
    {
        vec3 pos = ro + t*rd;
        float t = .5*tt;
        vec2 tuv;
        map( pos, tuv );
        vec2 ttuv = (tuv/3.1415)/2.0 + 0.5;
    
        vec2 tpa = vec2(8.0, 24.0);
        float pa = sin(tpa.x*tuv.y + tpa.y*tuv.x); 
         
        
        float pa_id = hash11(floor(mod(tpa.x*ttuv.y+tpa.y*ttuv.x,tpa.x))/(tpa.x-1.0));
        float grad = 2.0*mod(tpa.x*ttuv.x+tpa.y*ttuv.y,tpa.x)/tpa.x - 1.0;
        float cars = smoothstep(-.002,.002,sin(grad + sign(hash11(pa_id)-.5)*6.0*t*pa_id) - 
                                hash11(pa_id));
        float distb = smoothstep(-0.05,0.05,sin(tpa.x*tuv.x + tpa.y*tuv.y + sign(hash11(pa_id)-.5)*20.0*t) +
                                 .9 + .1*sin(40.0*ttuv.y));
        
        col = vec3(distb);
        col = cars * distb * vec3(smoothstep(-0.5,.05,pa));
    }

    color = vec4(u.c * col, 1.0);
}
