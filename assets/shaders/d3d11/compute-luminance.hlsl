#include "_constants.hlsli"
#include "_cbuffers.hlsli"
#include "_layouts.hlsli"
#include "_functions.hlsli"

CBUFFER_VIEW_MATRICES(b0);

Texture2D raw_output : register(t0);
RWByteAddressBuffer luminances : register(u4);

groupshared uint block_luminances[BLOCK_WIDTH * BLOCK_HEIGHT];

[numthreads(BLOCK_WIDTH, BLOCK_HEIGHT, 1)]
void compute_luminance(uint3 gti : SV_GroupThreadID, uint3 dti : SV_DispatchThreadID) {
    uint index = BLOCK_WIDTH * dti.y + dti.x;
    block_luminances[index] = luminance(raw_output[dti.xy].rgb);

    GroupMemoryBarrierWithGroupSync();
    if (gti.x + gti.y > 0) {
        return;
    }

    uint this_luminance = 0;
    for (uint i = 0; i < BLOCK_WIDTH * BLOCK_HEIGHT; i++) {
        this_luminance += uint(block_luminances[i] * 8);
    }

    luminances.InterlockedAdd(0, this_luminance);
}
