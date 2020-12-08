#include "common.hlsli"

cbuffer ViewMatrices : register(b0) {
    float4x4 view;
    float4x4 projection;
    float4x4 view_inv;
    float4x4 projection_inv;
    float4 screen_time;
};

cbuffer ModelMatrices : register(b1) {
    float4x4 model;
};

Texture2D albedo : register(t0);

GBufferInput vertex_main(VsInput input) {
    GBufferInput output;
    output.position = mul(projection, mul(view, mul(model, float4(input.position, 1.0))));
    output.world_position = mul(model, float4(input.position, 1.0));
    output.world_normal = (mul(model, float4(input.normal, 0.0)) + 1.0) / 2.0;
    output.uv = input.uv;
    return output;
}

GBufferOutput pixel_main(GBufferInput input) {
    // 復元されたワールド位置
    float4 position = float4(
        ((input.position.x / screen_time.x) * 2.0 - 1.0) * input.position.w,
        ((input.position.y / screen_time.y) * 2.0 - 1.0) * -input.position.w,
        input.position.z * input.position.w,
        input.position.w
    );
    position = mul(view_inv, mul(projection_inv, position));

    GBufferOutput output;
    // output.albedo = float4(abs(position - input.world_position).xyz, 1.0);
    output.albedo = float4(albedo.Sample(globalSampler, input.uv).rgb, 1.0);
    output.world_position = float4(1.0, 1.0, 1.0, 1.0);
    output.world_normal = float4(input.world_normal.rgb, 1.0);

    return output;
}

