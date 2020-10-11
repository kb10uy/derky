#version 450

layout(binding = 1) uniform atomic_uint env_atomic_counters[8];
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
    float red = max(min(0.9999, albedo.r), 0.0);
    uint index = uint(red * 64.0);
    if (red < 0.125) {
        atomicCounterIncrement(env_atomic_counters[0]);
    } else if (red < 0.250) {
        atomicCounterIncrement(env_atomic_counters[1]);
    } else if (red < 0.375) {
        atomicCounterIncrement(env_atomic_counters[2]);
    } else if (red < 0.500) {
        atomicCounterIncrement(env_atomic_counters[3]);
    } else if (red < 0.625) {
        atomicCounterIncrement(env_atomic_counters[4]);
    } else if (red < 0.750) {
        atomicCounterIncrement(env_atomic_counters[5]);
    } else if (red < 0.875) {
        atomicCounterIncrement(env_atomic_counters[6]);
    } else {
        atomicCounterIncrement(env_atomic_counters[7]);
    }

    out_albedo = vec4(albedo);
    out_position = vec4(v_position, 1.0);
    out_world_normal = vec4(normalize(v_normal), 0.0);
}
