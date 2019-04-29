#version 450
#extension GL_ARB_separate_shader_objects : enable


layout(constant_id = 0) const float scale = 1.0f;

layout(location = 0) in vec3 a_Position;
layout(location = 1) in vec2 a_TexCoord;

layout(location = 0) out vec2 vs_TexCoord;

layout(set = 1, binding = 0) uniform Camera {
    mat4 mvp;
} camera;

out gl_PerVertex {
    vec4 gl_Position;
};

void main() {
    vs_TexCoord = a_TexCoord;

    gl_Position = camera.mvp * vec4(scale * a_Position, 1.0);
}