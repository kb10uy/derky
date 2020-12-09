#include "common.hlsli"

Texture2D world_position : register(t0);
Texture2D world_normal : register(t1);

LightingInput vertex_screen(VsInput input) {
    LightingInput output;
    output.position = float4(input.position, 1.0);
    output.uv = input.uv;
    return output;
}

LightingOutput pixel_directional(LightingInput input) {
    LightingOutput output;
    output.intensity = vec4(1.0, 0.0, 1.0, 1.0);

    return output;
}
