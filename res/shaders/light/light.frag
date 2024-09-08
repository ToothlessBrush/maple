#version 330 core

out vec4 color;

in vec4 v_Color;

// ran for ever pixel
void main() {
    color = v_Color;
}