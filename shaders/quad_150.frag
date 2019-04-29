#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec2 vs_TexCoord;

layout(location = 0) out vec4 Target0;

layout(set = 0, binding = 0) uniform texture2D u_Texture;
layout(set = 0, binding = 1) uniform sampler u_Sampler;

layout(set = 2, binding = 0) uniform ubo_Color {
    vec4 col;
} quadColor;

void main() {
    Target0 = texture(sampler2D(u_Texture, u_Sampler), vs_TexCoord) * quadColor.col;
}