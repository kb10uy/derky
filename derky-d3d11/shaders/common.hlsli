#ifndef DERKY_COMMON
#define DERKY_COMMON

struct VsInput {
    float3 position: POSITION0;
    float3 normal: NORMAL0;
    float2 uv: TEXCOORD0;
};

struct PsInput {
    float4 position: SV_POSITION;
    float2 uv: TEXCOORD0;
};

struct PsOutput {
    float4 color: SV_TARGET;
};

#endif
