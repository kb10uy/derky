#include "_cbuffers.hlsli"
#include "_constants.hlsli"
#include "_layouts.hlsli"

CBUFFER_VIEW_MATRICES(b0);

Texture2D albedo : register(t0);
Texture2D world_position : register(t1);
Texture2D world_normal : register(t2);
Texture2D depth : register(t4);
Texture2D lighting : register(t5);

RWByteAddressBuffer luminances : register(u4);

float luminance(float3 color) {
    return dot(color, VEC_RGB_LUMA);
}

float aces_filmic(float value) {
    return clamp((value * (value * 2.51 + 0.03)) / (value * (value * 2.43 + 0.59) + 0.14), 0.0, 1.0);
}

CompositionInput vertex_main(VsInput input) {
    CompositionInput output;
    output.position = float4(input.position, 1.0);
    output.uv = input.uv;

    return output;
}

CompositionOutput pixel_main(CompositionInput input) {
    float3 albedo_color = albedo.Sample(globalSampler, input.uv).rgb;
    float3 light_color = lighting.Sample(globalSampler, input.uv).rgb;
    float3 raw_color = albedo_color * light_color;

    uint this_luminance = uint(luminance(raw_color) * 8.0);
    luminances.InterlockedAdd(0, this_luminance);

    float prev_luminance_average = screen_time.w / (8.0 * 1280.0 * 720.0);
    float3 exposure_color = raw_color * (0.18 / prev_luminance_average);
    float3 final_color = exposure_color;

    final_color = float3(aces_filmic(final_color.r), aces_filmic(final_color.g), aces_filmic(final_color.b));
    final_color = float3(pow(final_color.r, 1.0 / 2.2), pow(final_color.g, 1.0 / 2.2), pow(final_color.b, 1.0 / 2.2));

    CompositionOutput output;
    output.color = float4(final_color, 1.0);

    return output;
}
