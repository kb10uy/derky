#include "common.hlsli"

cbuffer Matrices : register(b0) {
    float4x4 model;
    float4x4 view;
    float4x4 projection;
};

Texture2D albedo : register(t0);
SamplerState albedoSampler : register(s0);

PsInput vertex_main(VsInput input) {
    PsInput output;

    // <del>以下は掛ける順が逆なのでダメっぽい</del>
    // [2020-12-02 13:46] そんなことはなくて、column-major で渡しているので column-major で計算していいらしい
    float4 position = mul(projection, mul(view, mul(model, float4(input.position, 1.0))));
    output.position = position;

    output.uv = input.uv;
    return output;
}

PsOutput pixel_main(PsInput input) {
    PsOutput output;
    output.color = float4(albedo.Sample(albedoSampler, input.uv).rgb, 1.0);
    return output;
}

