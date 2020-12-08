#ifndef DERKY_COMMON
#define DERKY_COMMON

// Vertex Shader の共通の入力
struct VsInput {
    float3 position: POSITION0;
    float3 normal: NORMAL0;
    float2 uv: TEXCOORD0;
};

// G-Buffer 用の構造体 -----------------------------------------

// Input
struct GBufferInput {
    float4 position: SV_Position;
    float4 world_position: COLOR0;
    float4 world_normal: NORMAL0;
    float2 uv: TEXCOORD0;
};

// Output
struct GBufferOutput {
    float4 albedo: SV_Target0;
    float4 world_position: SV_Target1;
    float4 world_normal: SV_Target2;
};

// Lighting Buffer 用の構造体 -----------------------------------

// Input
struct LightingInput {
    float4 position: SV_Position;
    float2 uv: TEXCOORD0;
};

// Output
struct LightingOutput {
    float4 intensity: SV_Target0;
};

// 合成用の構造体 -----------------------------------------------

// Pixel Shader のスクリーン全体の出力
struct CompositionInput {
    float4 position: SV_Position;
    float2 uv: TEXCOORD0;
};

// Pixel Shader のスクリーン全体の出力
struct CompositionOutput {
    float4 color: SV_Target0;
};

// 共通で仕様するデフォルトの SamplerState
SamplerState globalSampler : register(s0);

#endif
