#version 150 core

in vec2 v_TexCoord;

uniform sampler2D t_Diffuse;

out vec4 Target0;

void main() {
    vec4 diffuseColor = texture(t_Diffuse, v_TexCoord);
    Target0 = diffuseColor;
}