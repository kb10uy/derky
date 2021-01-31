#version 460

const mat3 MAT_RGB_YUV = mat3(
    0.2990,     0.5870,     0.1140,
   -0.1687,    -0.3313,     0.5000,
    0.5000,    -0.4187,    -0.0810
);

const mat3 MAT_YUV_RGB = mat3(
    1.0000,     0.0000,     1.4020,
    1.0000,    -0.3441,    -0.7141,
    1.0000,     1.7720,     0.0000
);

const vec3 VEC_RGB_LUMA = vec3(0.2990, 0.5870, 0.1140);

const float LUMINANCE_SAMPLING_SPARSENESS = 8.0;
const float WINDOW_WIDTH = 1280.0;
const float WINDOW_HEIGHT = 720.0;

// ----------------------------------------------------------------------------

layout(binding = 0) uniform atomic_uint next_luminance;
uniform float prev_luminance;
uniform sampler2D tex_unlit;
uniform sampler2D tex_lighting;

smooth in vec2 v_uv;

out vec4 color;

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

void process_luminance(vec2 uv) {
    vec2 scaled_uv = uv * vec2(WINDOW_WIDTH, WINDOW_HEIGHT);
    if (mod(scaled_uv.x, LUMINANCE_SAMPLING_SPARSENESS) < 1.0 &&  mod(scaled_uv.y, LUMINANCE_SAMPLING_SPARSENESS) < 1.0) {
        color = vec4(1.0, 0.0, 0.0, 1.0);
    }
}

void main() {
    // UV 調整
    vec2 uv_inv = vec2(v_uv.x, 1.0 - v_uv.y);
    vec2 scaled_uv = uv_inv * vec2(1280, 720);

    // バッファ取得
    vec3 unlit_color = texture(tex_unlit, uv_inv).rgb;
    vec3 light_color = texture(tex_lighting, v_uv).rgb;

    // HDR カラー
    vec3 raw_color = unlit_color * light_color;
    float raw_luminance = luminance(raw_color);

    // 輝度調整
    if (
        mod(scaled_uv.x, LUMINANCE_SAMPLING_SPARSENESS) < 1.0 &&
        mod(scaled_uv.y, LUMINANCE_SAMPLING_SPARSENESS) < 1.0
    ) {
        atomicCounterAdd(next_luminance, uint(raw_luminance * 8.0));
    }
    float prev_luminance_average =
        prev_luminance
        / (WINDOW_WIDTH * WINDOW_HEIGHT / pow(LUMINANCE_SAMPLING_SPARSENESS, 2.0))
        / 8.0;
    // prev_luminance_average = 0.18;


    // 調整
    vec3 exposure_color = raw_color * (0.18 / prev_luminance_average);
    vec3 final_color = exposure_color;

    // exp
    // float k = log(1.0 / 255.0) / 7.0;
    // final_color = 1.0 - exp(k * final_color);

    // Reinhard
    // final_color /= (1.0 + final_color);
    // final_color = vec3(pow(final_color.r, 1.0 / 2.2), pow(final_color.g, 1.0 / 2.2), pow(final_color.b, 1.0 / 2.2));

    // ACES Filmic
    final_color = vec3(aces_filmic(final_color.r), aces_filmic(final_color.g), aces_filmic(final_color.b));

    // OpenGL は線形出力に対して常に x^(1.0/2.2) のガンマ補正を行う？
    // final_color = vec3(pow(final_color.r, 1.0 / 2.2), pow(final_color.g, 1.0 / 2.2), pow(final_color.b, 1.0 / 2.2));

    color = vec4(final_color, 1.0);
}
