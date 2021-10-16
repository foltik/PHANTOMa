#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(early_fragment_tests) in;

layout(location = 0) in vec2 tex;
layout(location = 0) out vec4 color;

layout(set = 0, binding = 0) uniform texture2D img;
layout(set = 0, binding = 1) uniform sampler samp;

void main() {
    vec2 st = vec2(tex.x, 1.0 - tex.y);
    color = texture(sampler2D(img, samp), vec2(1.0 - st.x, st.y));
}
