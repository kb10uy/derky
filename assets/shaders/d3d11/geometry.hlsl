#include "common.hlsli"

cbuffer Matrices : register(b0) {
    float4x4 view;
    float4x4 projection;
};

cbuffer ModelMatrices : register(b1) {
    float4x4 model;
};

Texture2D albedo : register(t0);

GBufferInput vertex_main(VsInput input) {
    GBufferInput output;
    // <del>以下は掛ける順が逆なのでダメっぽい</del>
    // [2020-12-02 13:46] そんなことはなくて、column-major で渡しているので column-major で計算していいらしい
    float4 position = mul(projection, mul(view, mul(model, float4(input.position, 1.0))));
    output.position = position;
    output.uv = input.uv;
    return output;
}

GBufferOutput pixel_main(GBufferInput input) {
    GBufferOutput output;
    output.albedo = float4(albedo.Sample(globalSampler, input.uv).rgb, 1.0);
    output.world_normal = float4(0.5, 0.5, 1.0, 0.0);
    output.world_position = float4(1.0, 1.0, 1.0, 1.0);

    return output;
}

