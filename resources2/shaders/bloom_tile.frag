#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform texture2D imgs[];
layout(set = 0, binding = 1) uniform sampler samp;

layout(set = 1, binding = 0) uniform U {
    float fr;
} u;

vec3 jodieReinhardTonemap(vec3 c){
    float l = dot(c, vec3(0.2126, 0.7152, 0.0722));
    vec3 tc = c / (c + 1.0);
    return mix(c / (l + 1.0), tc, tc);
}

vec3 bloomTile(float lod, vec2 offset, vec2 uv){
    return texture(sampler2D(imgs[1], samp), uv * exp2(-lod) + offset).rgb;
}

vec3 getBloom(vec2 uv) {
    vec3 blur = vec3(0.0);

    blur = pow(bloomTile(2., vec2(0.0,0.0), uv),vec3(2.2))       	   	+ blur;
    blur = pow(bloomTile(3., vec2(0.3,0.0), uv),vec3(2.2)) * 1.3        + blur;
    blur = pow(bloomTile(4., vec2(0.0,0.3), uv),vec3(2.2)) * 1.6        + blur;
    blur = pow(bloomTile(5., vec2(0.1,0.3), uv),vec3(2.2)) * 1.9 	   	+ blur;
    blur = pow(bloomTile(6., vec2(0.2,0.3), uv),vec3(2.2)) * 2.2 	   	+ blur;

    float colorRange = 24.0;
    return blur * colorRange;
}

void main() {
    vec2 uv = vec2(tex.x, 1.0 - tex.y);

    //original
    vec3 col = texture(sampler2D(imgs[0], samp), uv).rgb;
    
    //add bloom
    vec3 bloom = getBloom(uv) * 0.02 * u.fr;
    bloom = jodieReinhardTonemap(bloom);
    col += bloom;
    
	color = vec4(col, 1.0);
}
