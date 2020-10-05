#version 450

// environmental uniforms
uniform mat4 mat_view;
uniform mat4 mat_projection;

// model uniforms
uniform mat4 mat_model;

// per vertex
in vec3 position;
in vec3 normal;
in vec2 uv;

smooth out vec3 v_normal;

void main() {
    v_normal = normalize(mat_model * vec4(normal, 0.0)).xyz;
    gl_Position = mat_projection * mat_view * mat_model * vec4(position, 1.0);
}
