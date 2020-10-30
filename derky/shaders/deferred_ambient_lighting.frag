#version 450

uniform vec3 light_ambient_intensity;

out vec4 color;

void main() {
    color = vec4(light_ambient_intensity, 1.0);
}
