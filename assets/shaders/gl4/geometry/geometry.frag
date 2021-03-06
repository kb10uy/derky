#version 450

uniform sampler2D material_albedo;

// per fragment
smooth in vec2 v_uv;
smooth in vec3 v_normal;
smooth in vec3 v_position;

out vec4 out_albedo;
out vec4 out_position;
out vec4 out_world_normal;

void main() {
    vec4 albedo = texture(material_albedo, v_uv);

    if (albedo.a < 0.5) {
        discard;
    }

    out_albedo = vec4(albedo);
    out_position = vec4(v_position, 1.0);
    out_world_normal = vec4(normalize(v_normal), 0.0);
}
