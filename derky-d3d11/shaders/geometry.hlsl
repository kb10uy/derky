#include "common.hlsli"

cbuffer Matrices : register(b0) {
    float4x4 model;
    float4x4 view;
    float4x4 projection;
};

PsInput vertex_main(VsInput input) {
    PsInput output;
    // output.position = mul(projection, mul(view, mul(model, float4(input.position, 1.0))));
    output.position = mul(mul(mul(float4(input.position, 1.0), model), view), projection);
    output.uv = input.uv;
    return output;
}

PsOutput pixel_main(PsInput input) {
    PsOutput output;
    output.color = float4(1.0, 1.0, 0.0, 1.0);
    return output;
}

