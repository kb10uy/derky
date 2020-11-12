#version 450

// environmental uniforms
uniform mat4 env_view_matrix;
uniform mat4 env_projection_matrix;

// model uniforms
uniform mat4 model_matrix;

// per vertex
in vec3 position;
in vec3 normal;
in vec2 uv;

smooth out vec2 v_uv;
smooth out vec3 v_normal;
smooth out vec3 v_position;

void main() {
    v_uv = vec2(uv.x, 1.0 - uv.y);
    v_normal = normalize(model_matrix * vec4(normal, 0.0)).xyz;
    v_position = (model_matrix * vec4(position, 1.0)).xyz;
    gl_Position = env_projection_matrix * env_view_matrix * model_matrix * vec4(position, 1.0);
}
