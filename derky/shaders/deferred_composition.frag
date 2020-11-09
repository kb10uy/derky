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

float aces_filmic(float value) {
    return clamp((value * (value * 2.51 + 0.03)) / (value * (value * 2.43 + 0.59) + 0.14), 0.0, 1.0);
}

void main() {
    vec2 uv_inv = vec2(v_uv.x, 1.0 - v_uv.y);
    vec3 unlit_color = texture(tex_unlit, uv_inv).rgb;
    vec3 light_color = texture(tex_lighting, v_uv).rgb;

    // HDR カラー
    vec3 raw_color = unlit_color * light_color;
    float raw_luminance = luminance(raw_color);

    /*
    // 次フレーム用の輝度
    uint luma_index = uint(min(raw_luminance * 8.0, 7.99999));
    switch (luma_index) {
        case 7:
            atomicCounterIncrement(next_luminance);
        case 6:
            atomicCounterIncrement(next_luminance);
        case 5:
            atomicCounterIncrement(next_luminance);
        case 4:
            atomicCounterIncrement(next_luminance);
        case 3:
            atomicCounterIncrement(next_luminance);
        case 2:
            atomicCounterIncrement(next_luminance);
        case 1:
            atomicCounterIncrement(next_luminance);
        case 0:
            atomicCounterIncrement(next_luminance);
    }
    */

    // 今フレームの露出
    // yuv_color.x *= prev_luminance_average * 0.18;
    // yuv_color.x = pow(yuv_color.x, 1.0 / 2.2);
    float prev_luminance_average = 2.0; // prev_luminance / (1280.0 * 720.0 * 8.0);
    vec3 exposure_color = raw_color * (0.18 / prev_luminance_average);

    // オレオレ謎関数 No.1
    // out_color = atan(5 * (out_color - 0.4)) / 2.6 + 0.42;

    // exp
    // float k = log(1.0 / 255.0) / 7.0;
    // out_color = 1.0 - exp(k * out_color);

    // Reinhard
    // out_color /= (1.0 + out_color);

    // ACES Filmic
    // out_color = vec3(aces_filmic(out_color.r), aces_filmic(out_color.g), aces_filmic(out_color.b));

    vec3 final_color = raw_color;

    if (v_uv.y < 0.25) {
        // Raw
        final_color = raw_color;
    } else if (v_uv.y < 0.50) {
        // Reinhard
        final_color = exposure_color / (1.0 + exposure_color);
        // ガンマ補正
        final_color = vec3(pow(final_color.r, 1.0 / 2.2), pow(final_color.g, 1.0 / 2.2), pow(final_color.b, 1.0 / 2.2));
    } else if (v_uv.y < 0.75) {
        // ACES
        final_color = vec3(aces_filmic(exposure_color.r), aces_filmic(exposure_color.g), aces_filmic(exposure_color.b));
        // ガンマ補正
        final_color = vec3(pow(final_color.r, 1.0 / 2.2), pow(final_color.g, 1.0 / 2.2), pow(final_color.b, 1.0 / 2.2));
    } else {
        // オレオレ #1
        final_color = atan(5 * (exposure_color - 0.4)) / 2.6 + 0.42;
        // ガンマ補正
        final_color = vec3(pow(final_color.r, 1.0 / 2.2), pow(final_color.g, 1.0 / 2.2), pow(final_color.b, 1.0 / 2.2));
    }

    color = vec4(final_color, 1.0);
}
