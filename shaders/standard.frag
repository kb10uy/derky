#version 450

flat in vec3 v_normal;

out vec4 color;

void main() {
    color = vec4(v_normal.rgb, 1.0);
}
