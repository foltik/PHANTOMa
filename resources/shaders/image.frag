#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform texture2D img;
layout(set = 0, binding = 1) uniform sampler samp;
layout(set = 0, binding = 2) uniform Uniform {
    vec2 pos;
    vec2 scale;
} u;

void main() {
    vec2 t = vec2(tex.x, 1 - tex.y);// + (u.pos / vec2(1920.0, 1080.0));
    color = texture(sampler2D(img, samp), t * u.scale);
}
