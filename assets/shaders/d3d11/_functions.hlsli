#ifndef DERKY_FUNCTIONS
#define DERKY_FUNCTIONS

#include "_constants.hlsli"

float luminance(float3 color) {
    return dot(color, VEC_RGB_LUMA);
}

float aces_filmic(float value) {
    return clamp((value * (value * 2.51 + 0.03)) / (value * (value * 2.43 + 0.59) + 0.14), 0.0, 1.0);
}

#endif
