#version 450

uniform mat4 mat_model;
uniform mat4 mat_view;
uniform mat4 mat_projection;
uniform vec3 lit_directional;

flat in vec3 v_normal;

out vec4 color;

void main() {
    float luminance = dot(-v_normal, lit_directional);
    color = vec4(luminance, luminance, luminance, 1.0);
}
