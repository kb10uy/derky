/*
 * Constant Buffer definitions
 */

#ifndef DERKY_CBUFFER
#define DERKY_CBUFFER

// cbuffer ViewMatrices
#define CBUFFER_VIEW_MATRICES(slot) \
    cbuffer ViewMatrices : register(slot) { \
        float4x4 view; \
        float4x4 projection; \
        float4x4 view_inv; \
        float4x4 projection_inv; \
        float4 screen_time; \
    }

// cbuffer ModelData
#define CBUFFER_MODEL_DATA(slot) \
    cbuffer ModelData : register(slot) { \
        float4x4 model; \
    }

// cbuffer DirectionalLight
#define CBUFFER_DIRECTIONAL_LIGHT(slot) \
    cbuffer DirectionalLight : register(slot) { \
        float4 directional_intensity; \
        float4 directional_direction; \
    }

// cbuffer PointLight
#define CBUFFER_POINT_LIGHT(slot) \
    cbuffer PointLight : register(slot) { \
        float4 point_intensity; \
        float4 point_position; \
    }

// cbuffer ImageLight
#define CBUFFER_IMAGE_LIGHT(slot) \
    cbuffer ImageLight : register(slot) { \
        float4 image_intensity; \
    }

#endif
