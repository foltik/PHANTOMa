#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 tex;
layout(location = 2) in vec3 norm;

layout(location = 0) out vec4 color;

#define MAX 32

struct Light {
    uint ty;
    float intensity;
    float range;
    float angle;
    vec3 color;
};

layout(set = 1, binding = 0) uniform Lights {
    Light l[MAX];
} lights;

layout(set = 1, binding = 1) uniform LightTransforms {
    mat4 m[MAX];
} transforms;

layout(set = 1, binding = 2) uniform LightCount {
    int n;
} count;

layout(set = 2, binding = 0) uniform texture2D img;
layout(set = 2, binding = 1) uniform sampler samp;
layout(set = 2, binding = 2) uniform Material {
    vec2 scale;
    uint unlit;
} mat;

vec3 calc_point_light(Light light, mat4 transform, vec3 col) {
    vec3 normal = normalize(norm);
    vec3 light_pos = vec3(transform * vec4(vec3(0.0), 1.0));
    vec3 to_light = light_pos - pos;

    vec3 light_dir = normalize(to_light);
    vec3 reflect_dir = reflect(-light_dir, normal);

    float f_ambient = 0.0;
    float f_diffuse = max(dot(normal, light_dir), 0.0);
    float f_specular = pow(max(dot(normalize(-pos), reflect_dir), 0.0), 128);

    vec3 ambient  = light.color * f_ambient;
    vec3 diffuse  = light.color * f_diffuse;
    vec3 specular = light.color * f_specular;

    float d = length(light_pos - pos);

    // PBR???
    // float atten_dist_sq = dot(to_light, to_light);
    // float atten_fac = atten_dist_sq * light.range * light.range;
    // float atten_smooth = max(1.0 - atten_fac * atten_fac, 0.0);
    // float f_atten = (atten_smooth * atten_smooth) / max(atten_dist_sq, 1e-4);

    float r_smooth = 2.0;
    float f_rad = smoothstep(r_smooth, 0.0, d - light.range + r_smooth);
    float f_atten = 1.0 / (1.0 + 0.1*d + 0.01*d*d);

    return col * (ambient + diffuse + specular) * f_atten * f_rad;
}

void main() {
    // vec3 col = vec3(0.8, 0.057, 0.162);
    vec3 col = texture(sampler2D(img, samp), tex * mat.scale).rgb;

    if (mat.unlit != 0) {
        color = vec4(col, 1.0);
    } else {
        vec3 res = vec3(0.0);
        for (int i = 0; i < count.n; i++)
            res += calc_point_light(lights.l[i], transforms.m[i], col);

        color = vec4(res, 1.0);
    }
}