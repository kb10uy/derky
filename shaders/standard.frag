#version 450

uniform mat4 mat_model;
uniform mat4 mat_view;
uniform mat4 mat_projection;

flat in vec3 v_normal;

out vec4 color;

void main() {
    color = vec4(v_normal.rgb, 1.0);
}
