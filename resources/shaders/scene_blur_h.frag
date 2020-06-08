#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform texture2D img;
layout(set = 0, binding = 1) uniform sampler samp;

vec4 blur13(vec2 amt) {
    vec4 color = vec4(0.0);
    vec2 off1 = vec2(1.411764705882353) * amt;
    vec2 off2 = vec2(3.2941176470588234) * amt;
    vec2 off3 = vec2(5.176470588235294) * amt;
    color += texture(sampler2D(img, samp), tex) * 0.1964825501511404;
    color += texture(sampler2D(img, samp), tex + off1) * 0.2969069646728344;
    color += texture(sampler2D(img, samp), tex - off1) * 0.2969069646728344;
    color += texture(sampler2D(img, samp), tex + off2) * 0.09447039785044732;
    color += texture(sampler2D(img, samp), tex - off2) * 0.09447039785044732;
    color += texture(sampler2D(img, samp), tex + off3) * 0.010381362401148057;
    color += texture(sampler2D(img, samp), tex - off3) * 0.010381362401148057;
    return color;
}

void main() {
    color = blur13(vec2(-1.0 / 1920.0, 0.0));
}
