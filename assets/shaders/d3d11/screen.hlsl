#include "common.hlsli"

Texture2D albedo : register(t0);
SamplerState albedoSampler : register(s0);

CompositionInput vertex_main(VsInput input) {
    CompositionInput output;
    output.position = float4(input.position, 1.0);
    output.uv = input.uv;

    return output;
}

CompositionOutput pixel_main(CompositionInput input) {
    CompositionOutput output;
    output.color = float4(albedo.Sample(albedoSampler, input.uv).rgb, 1.0);

    return output;
}
