#include "_layouts.hlsli"

Texture2D albedo : register(t0);
Texture2D world_position : register(t1);
Texture2D world_normal : register(t2);
Texture2D lighting : register(t5);
RWByteAddressBuffer luminances : register(u4);

CompositionInput vertex_main(VsInput input) {
    CompositionInput output;
    output.position = float4(input.position, 1.0);
    output.uv = input.uv;

    return output;
}

CompositionOutput pixel_main(CompositionInput input) {
    float3 albedo_color = albedo.Sample(globalSampler, input.uv).rgb;
    float3 light_color = lighting.Sample(globalSampler, input.uv).rgb;

    CompositionOutput output;
    output.color = float4(albedo_color * light_color, 1.0);

    uint luminance = uint(light_color.r * 8.0);
    luminances.InterlockedAdd(0, luminance);

    return output;
}
