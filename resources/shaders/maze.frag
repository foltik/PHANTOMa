#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 1) in vec3 norm;
layout(location = 2) in vec3 pos;

layout(location = 0) out vec4 color;


struct PointLight {
    vec3 pos;

    float constant;
    float linear;
    float quadratic;

    float ambient;
    float diffuse;
    float specular;
};

layout(set = 1, binding = 0) uniform texture2D img;
layout(set = 1, binding = 1) uniform sampler samp;
layout(set = 1, binding = 2) uniform Uniforms {
    PointLight light;
} u;

const PointLight light = {
    vec3(0.0, 0.0, 2.0),
    1.0,
    1.0,
    0.5,
    1.0,
    1.0,
    1.0,
};

const PointLight light2 = {
    vec3(0.0, 0.0, 1.0),
    100.0,
    1.0,
    8.0,
    10.0,
    1.0,
    1.0,
};

vec3 CalcPointLight(PointLight light, vec3 normal, vec3 fragPos, vec3 viewDir, vec3 color)
{
    vec3 lightDir = normalize(light.pos - fragPos);
    // diffuse shading
    float diff = max(dot(normal, lightDir), 0.0);
    // specular shading
    vec3 reflectDir = reflect(-lightDir, normal);
    float shininess = 0.5;
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), shininess);
    // attenuation
    float distance    = length(light.pos - fragPos);
    float attenuation = 1.0 / (light.constant + light.linear * distance +
                               light.quadratic * (distance * distance));
    // combine results
    vec3 ambient  = vec3(light.ambient)  * color;
    vec3 diffuse  = vec3(light.diffuse)  * diff * color;
    vec3 specular = vec3(light.specular) * spec * color;
    ambient  *= attenuation;
    diffuse  *= attenuation;
    specular *= attenuation;
    return ambient + diffuse + specular;
}

void main() {
    vec3 mat = texture(sampler2D(img, samp), tex).rgb;
    vec3 c = vec3(0.0);

    float ambient = 0.001;
    c += ambient * mat;

    vec3 n = normalize(norm);
    vec3 viewPos = vec3(0.0, 0.75, -2.0);
    vec3 viewDir = normalize(viewPos - pos);

    c += CalcPointLight(u.light, n, pos, viewDir, mat);
    //c += CalcPointLight(light, n, pos, viewDir, mat);
    //c += CalcPointLight(light2, n, pos, viewDir, mat);

    color = vec4(c, 1.0);
}
