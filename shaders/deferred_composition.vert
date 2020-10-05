#version 450

uniform mat4 mat_screen;

in vec4 position;
in vec2 uv;

smooth out vec2 v_uv;

void main() {
    v_uv = uv;
    gl_Position = mat_screen * position;
}
