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

    vec4 attenuation;
};

layout(set = 1, binding = 0) uniform texture2D img;
layout(set = 1, binding = 1) uniform sampler samp;
layout(set = 1, binding = 2, std140) uniform Uniforms {
    PointLight light;
    vec3 eye;
} u;

vec3 CalcPointLight(PointLight light, vec3 normal, vec3 fragPos, vec3 viewDir, vec3 color)
{
    vec3 lightDir = normalize(light.pos.xyz - fragPos);
    // diffuse shading
    float diff = max(dot(normal, lightDir), 0.0);
    // specular shading
    vec3 reflectDir = reflect(-lightDir, normal);
    float shininess = 0.5;
    float spec = pow(max(dot(viewDir, reflectDir), 0.0), shininess);
    // attenuation
    float distance    = length(light.pos.xyz - fragPos);
    float attenuation = 1.0 / (light.attenuation.z + light.attenuation.y * distance +
                               light.attenuation.x * (distance * distance));
    // combine results
    vec3 ambient  = light.ambient.xyz * color;
    vec3 diffuse  = light.diffuse.xyz * diff * color;
    vec3 specular = light.specular.xyz * spec * color;
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
    vec3 viewDir = normalize(u.eye - pos);
    c += CalcPointLight(u.light, n, pos, viewDir, mat);

    color = vec4(c, 1.0);
    color = vec4(1.0);
}
