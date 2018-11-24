#version 150 core

in vec3 a_Position;
in vec2 a_TexCoord;

uniform u_Camera {
    mat4 u_Projection;
    mat4 u_View;
};

out vec2 v_TexCoord;
out vec3 v_Position;

void main() {
    v_TexCoord = a_TexCoord;
    v_Position = a_Position;

    gl_Position = u_Projection * u_View * vec4(a_Position, 1.0);
}