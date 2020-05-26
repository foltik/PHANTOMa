#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 1) in vec3 norm;
layout(location = 2) in vec3 pos;

layout(location = 0) out vec4 color;


struct PointLight {
    vec4 pos;

    vec4 ambient;
    vec4 diffuse;
    vec4 specular;

    vec4 atten;
};

layout(set = 0, binding = 0) uniform LightsInfo {
    vec4 info;
} lights;
layout(set = 0, binding = 1) uniform PointLights {
    PointLight lights[16];
} point_lights;

layout(set = 1, binding = 1) uniform Camera {
    vec4 eye;
} cam;

layout(set = 3, binding = 1) uniform Material {
    vec4 ambient;
    vec4 diffuse;
    vec4 specular;
    vec4 emissive;
    vec4 extra;
} mat;
layout(set = 3, binding = 2) uniform sampler samp;
layout(set = 3, binding = 3) uniform texture2D img;

struct MaterialSample {
    vec3 ambient;
    vec3 diffuse;
    vec3 specular;
};

vec3 calc_point_light(PointLight light, MaterialSample mat, vec3 col, vec3 dir, vec3 pos, vec3 norm) {
    vec3 light_dir = normalize(light.pos.xyz - pos);
    vec3 reflect_dir = reflect(-light_dir, norm);

    float f_ambient = 1.0;
    float f_diffuse = max(dot(norm, light_dir), 0.0);
    float f_specular = pow(max(dot(dir, reflect_dir), 0.0), 32); // TODO change shininess 0.5 to sample

    vec3 ambient  = mat.ambient  * light.ambient.rgb  * f_ambient;
    vec3 diffuse  = mat.diffuse  * light.diffuse.rgb  * f_diffuse;
    vec3 specular = mat.specular * light.specular.rgb * f_specular;

    float d = length(light.pos.xyz - pos);
    float atten = 1.0 / (light.atten.z + light.atten.y * d + light.atten.x * (d * d));

    return atten * col * (ambient + diffuse + specular);
}

void main() {
    vec3 col = texture(sampler2D(img, samp), tex).rgb;

    MaterialSample s = MaterialSample(
        mat.ambient.rgb,
        mat.diffuse.rgb,
        mat.specular.rgb
    );

    vec3 res = vec3(0.0);

    float ambient = lights.info[0];
    res += ambient * col;

    vec3 dir = normalize(cam.eye.xyz - pos);

    for (int i = 0; i < int(lights.info[1]); i++)
        res += calc_point_light(point_lights.lights[i], s, col, dir, pos, norm);

    color = vec4(res, 1.0);
}