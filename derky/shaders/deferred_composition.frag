#version 460

layout(binding = 0) uniform atomic_uint next_luminance;
uniform float prev_luminance;
uniform sampler2D tex_unlit;
uniform sampler2D tex_lighting;

smooth in vec2 v_uv;

out vec4 color;

// Strictly, YPbPr
mat3 MAT_RGB_YUV = mat3(
    0.2990,     0.5870,     0.1140,
   -0.1687,    -0.3313,     0.5000,
    0.5000,    -0.4187,    -0.0810
);

mat3 MAT_YUV_RGB = mat3(
    1.0000,     0.0000,     1.4020,
    1.0000,    -0.3441,    -0.7141,
    1.0000,     1.7720,     0.0000
);

vec3 VEC_RGB_LUMA = vec3(0.2990, 0.5870, 0.1140);

float luminance(vec3 color) {
    return dot(color, VEC_RGB_LUMA);
}

vec3 rgb2yuv(vec3 rgb_color) {
    return MAT_RGB_YUV * rgb_color;
}

vec3 yuv2rgb(vec3 yuv_color) {
    return MAT_YUV_RGB * yuv_color;
}

void main() {
    vec2 uv_inv = vec2(v_uv.x, 1.0 - v_uv.y);
    vec3 unlit_color = texture(tex_unlit, uv_inv).rgb;
    vec3 light_color = texture(tex_lighting, v_uv).rgb;

    // HDR カラー
    vec3 raw_color = unlit_color * light_color;
    float raw_luminance = luminance(raw_color);

    // 次フレーム用の輝度
    uint luma_index = uint(min(raw_luminance * 8.0, 7.99999));
    switch (luma_index) {
        case 0:
            atomicCounterIncrement(next_luminance);
        case 1:
            atomicCounterIncrement(next_luminance);
        case 2:
            atomicCounterIncrement(next_luminance);
        case 3:
            atomicCounterIncrement(next_luminance);
        case 4:
            atomicCounterIncrement(next_luminance);
        case 5:
            atomicCounterIncrement(next_luminance);
        case 6:
            atomicCounterIncrement(next_luminance);
        case 7:
            atomicCounterIncrement(next_luminance);
    }

    // 今フレームの露出
    // yuv_color.x *= prev_luminance_average * 0.18;
    // yuv_color.x = pow(yuv_color.x, 1.0 / 2.2);
    float prev_luminance_average = prev_luminance / (1280.0 * 720.0 * 8.0);
    vec3 out_color = raw_color * prev_luminance_average * 0.18;
    out_color /= (1.0 + raw_color);
    out_color = vec3(pow(out_color.r, 1.0 / 2.2), pow(out_color.g, 1.0 / 2.2), pow(out_color.b, 1.0 / 2.2));

    // Left/Right visualization
    if (v_uv.x < 0.5) {
        color = vec4(raw_color, 1.0);
    } else {
        color = vec4(out_color, 1.0);
    }
}
