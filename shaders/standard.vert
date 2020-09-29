#version 450

in vec3 position;
in vec2 texture_uv;

uniform mat4 mat_model;
uniform mat4 mat_view;
uniform mat4 mat_projection;


void main() {
    gl_Position = (mat_projection * mat_view * mat_model * vec4(position, 1.0)).xyz;
}
