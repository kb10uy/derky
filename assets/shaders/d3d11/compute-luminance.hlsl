#include "_constants.hlsli"
#include "_cbuffers.hlsli"
#include "_layouts.hlsli"
#include "_functions.hlsli"

CBUFFER_VIEW_MATRICES(b0);

Texture2D raw_output : register(t0);
RWByteAddressBuffer luminances : register(u4);

groupshared float block_luminances[BLOCK_WIDTH * BLOCK_HEIGHT];

[numthreads(BLOCK_WIDTH, BLOCK_HEIGHT, 1)]
void compute_luminance(uint3 gti : SV_GroupThreadID, uint3 dti : SV_DispatchThreadID) {
    uint block_index = BLOCK_WIDTH * gti.y + gti.x;
    float3 source = raw_output.Load(int3(dti.xy, 0)).rgb;
    block_luminances[block_index] = luminance(source);

    GroupMemoryBarrierWithGroupSync();
    if (gti.x + gti.y > 0) {
        return;
    }

    uint block_luminance_sum = 0;
    for (uint i = 0; i < BLOCK_WIDTH * BLOCK_HEIGHT; i++) {
        block_luminance_sum += uint(block_luminances[i] * 8);
    }
    luminances.InterlockedAdd(0, block_luminance_sum);
}
