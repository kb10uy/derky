#version 460

layout(binding = 0, offset = 0) uniform atomic_uint luma_next_0;
layout(binding = 0, offset = 4) uniform atomic_uint luma_next_1;
layout(binding = 0, offset = 8) uniform atomic_uint luma_next_2;
layout(binding = 0, offset = 12) uniform atomic_uint luma_next_3;
layout(binding = 0, offset = 16) uniform atomic_uint luma_next_4;
layout(binding = 0, offset = 20) uniform atomic_uint luma_next_5;
layout(binding = 0, offset = 24) uniform atomic_uint luma_next_6;
layout(binding = 0, offset = 28) uniform atomic_uint luma_next_7;
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

    if (out_luminance < 0.125) {
        atomicCounterIncrement(luma_next_0);
    } else if (out_luminance < 0.250) {
        atomicCounterIncrement(luma_next_1);
    } else if (out_luminance < 0.375) {
        atomicCounterIncrement(luma_next_2);
    } /* else if (out_luminance < 0.500) {
        atomicCounterIncrement(luma_prev_3);
    } else if (out_luminance < 0.625) {
        atomicCounterIncrement(luma_prev_4);
    } else if (out_luminance < 0.750) {
        atomicCounterIncrement(luma_prev_5);
    } else if (out_luminance < 0.875) {
        atomicCounterIncrement(luma_prev_6);
    } else {
        atomicCounterIncrement(luma_prev_7);
    }
    */
    // atomicCounterAdd(env_next_luminance, uint(out_luminance * 32.0));

    /*
    // Left/Right visualization
    if (v_uv.x < 0.5) {
        out_color /= exposure;
    }
    */

    // Normal output
    color = vec4(out_color, 1.0);
}
