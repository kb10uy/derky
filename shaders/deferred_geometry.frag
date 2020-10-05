#version 450

// environmental uniforms
uniform mat4 mat_view;
uniform mat4 mat_projection;
/*
uniform struct {
    vec3 directional_vector[4];
    vec3 directional_color[4];
} lights;
*/
// per fragment
smooth in vec3 v_normal;

out vec4 out_albedo;
out vec4 out_world_normal;

void main() {
    out_albedo = vec4(1.0, 1.0, 1.0, 1.0);
    out_world_normal = vec4(v_normal, 0.0);
}
