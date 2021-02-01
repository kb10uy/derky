/*
 * 定数群
 */

#ifndef DERKY_CONSTANTS
#define DERKY_CONSTANTS

static const float PI = 3.14159265;
static const float E = 2.718281828;

static const float3 VEC_RGB_LUMA = float3(0.2990, 0.5870, 0.1140);

static const float4x4 MAT_LUMINANCE_WEIGHTS = float4x4(
    256, 144, 64, 16,
    225, 121, 49, 9,
    196, 100, 36, 4,
    169, 81, 25, 1
) / 1496.0;

static const float4 VEC_DOT_SUM = float4(1, 1, 1, 1);

#endif
