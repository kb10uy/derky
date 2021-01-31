#include "_layouts.hlsli"
#include "_cbuffers.hlsli"

CBUFFER_VIEW_MATRICES(b0);
CBUFFER_MODEL_DATA(b1);

Texture2D albedo : register(t0);

GBufferInput vertex_main(VsInput input) {
    GBufferInput output;
    output.position = mul(projection, mul(view, mul(model, float4(input.position, 1.0))));
    output.world_position = mul(model, float4(input.position, 1.0));
    output.world_normal = mul(model, float4(input.normal, 0.0));
    output.uv = input.uv;
    return output;
}

GBufferOutput pixel_main(GBufferInput input) {
    GBufferOutput output;
    output.albedo = float4(albedo.Sample(globalSampler, input.uv).rgb, 1.0);
    output.world_position = input.world_position;
    output.world_normal = float4(input.world_normal.rgb, 1.0);

    return output;
}

