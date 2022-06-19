#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform texture2D imgs[];
layout(set = 0, binding = 1) uniform sampler samp;

layout(set = 1, binding = 0) uniform U {
    float w;
    float h;
} u;

vec3 makeBloom(float lod, vec2 offset, vec2 bCoord){
    vec2 pixelSize = 1.0 / vec2(u.w, u.h);

    offset += pixelSize;

    float lodFactor = exp2(lod);

    vec3 bloom = vec3(0.0);
    vec2 scale = lodFactor * pixelSize;

    vec2 coord = (bCoord.xy-offset)*lodFactor;
    float totalWeight = 0.0;

    if (any(greaterThanEqual(abs(coord - 0.5), scale + 0.5)))
        return vec3(0.0);

    for (int i = -5; i < 5; i++) {
        for (int j = -5; j < 5; j++) {

            float wg = pow(1.0-length(vec2(i,j)) * 0.125,6.0);

            bloom = pow(texture(sampler2D(imgs[0], samp),vec2(i,j) * scale + lodFactor * pixelSize + coord, lod).rgb,vec3(2.2))*wg + bloom;
            totalWeight += wg;

        }
    }

    bloom /= totalWeight;

    return bloom;
}

void main() {
    vec2 uv = vec2(tex.x, 1.0 - tex.y);
    
	vec3 blur = makeBloom(2.,vec2(0.0,0.0), uv);
		blur += makeBloom(3.,vec2(0.3,0.0), uv);
		blur += makeBloom(4.,vec2(0.0,0.3), uv);
		blur += makeBloom(5.,vec2(0.1,0.3), uv);
		blur += makeBloom(6.,vec2(0.2,0.3), uv);

    color = vec4(pow(blur, vec3(1.0 / 2.2)),1.0);
}
