#version 460

layout(binding = 0) uniform atomic_uint env_prev_luminance;
layout(binding = 1) uniform atomic_uint env_next_luminance;
uniform sampler2D tex_unlit;
uniform sampler2D tex_lighting;

smooth in vec2 v_uv;

out vec4 color;

float luminance(vec3 color) {
    return dot(color, vec3(0.299, 0.587, 0.114));
}

void main() {
    vec2 uv_inv = vec2(v_uv.x, 1.0 - v_uv.y);
    vec3 unlit_color = texture(tex_unlit, uv_inv).rgb;
    vec3 light_color = texture(tex_lighting, v_uv).rgb;
    float exposure = 1.0; // pow(max(max(light_color.r, light_color.g), light_color.b), 1.4);

    vec3 out_color = unlit_color * light_color * exposure;
    float out_luminance = luminance(out_color);

    atomicCounterAdd(env_next_luminance, uint(out_luminance * 32.0));

    /*
    // Left/Right visualization
    if (v_uv.x < 0.5) {
        out_color /= exposure;
    }
    */

    // Normal output
    color = vec4(out_color, 1.0);
}
