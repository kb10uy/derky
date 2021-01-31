#include "_cbuffers.hlsli"
#include "_constants.hlsli"
#include "_layouts.hlsli"

CBUFFER_VIEW_MATRICES(b0);

Texture2D albedo : register(t0);
Texture2D world_position : register(t1);
Texture2D world_normal : register(t2);
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
    float3 source_color = albedo_color * light_color;

    uint this_luminance = uint(luminance(source_color) * 8.0);
    luminances.InterlockedAdd(0, this_luminance);

    float previous_luminance = screen_time.w / (8.0 * 1280.0 * 720.0);
    float exposure_multiplier = 0.18 / previous_luminance;
    float4 output_color = float4(
        pow(aces_filmic(source_color.r * exposure_multiplier), 1.0 / 2.2),
        pow(aces_filmic(source_color.g * exposure_multiplier), 1.0 / 2.2),
        pow(aces_filmic(source_color.b * exposure_multiplier), 1.0 / 2.2),
        1.0
    );

    CompositionOutput output;
    output.color = output_color;

    return output;
}
