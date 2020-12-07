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
    float4 position: SV_POSITION;
    float2 uv: TEXCOORD0;
};

// Output
struct GBufferOutput {
    float4 albedo: SV_TARGET0;
    float4 world_position: SV_TARGET1;
    float4 world_normal: SV_TARGET2;
};

// Lighting Buffer 用の構造体 -----------------------------------

// Input
struct LightingInput {

};

// Output
struct LightingOutput {
    float4 intensity: SV_TARGET;
};

// 合成用の構造体 -----------------------------------------------

// Pixel Shader のスクリーン全体の出力
struct CompositionInput {
    float4 position: SV_POSITION;
    float2 uv: TEXCOORD0;
};

// Pixel Shader のスクリーン全体の出力
struct CompositionOutput {
    float4 color: SV_TARGET;
};

// 共通で仕様するデフォルトの SamplerState
SamplerState globalSampler : register(s0);

#endif
