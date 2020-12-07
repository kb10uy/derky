#include "common.hlsli"

LightingInput vertex_screen(VsInput input) {
    LightingInput output;

    return output;
}

LightingOutput pixel_directional(LightingInput input) {
    LightingOutput output;
    output.intensity = vec4(1.0, 0.0, 1.0, 1.0);

    return output;
}
