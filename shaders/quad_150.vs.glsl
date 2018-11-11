#version 150 core

in vec2 a_Position;
in vec2 a_TexCoord;

uniform u_Camera {
    mat4 u_Projection;
    mat4 u_View;
};

out vec2 v_TexCoord;

void main() {
    v_TexCoord = a_TexCoord;
    gl_Position = u_Projection * u_View * vec4(a_Position, 0.0, 1.0);
}