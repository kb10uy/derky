#include "_layouts.hlsli"
#include "_cbuffers.hlsli"
#include "_constants.hlsli"

CBUFFER_VIEW_MATRICES(b0);
CBUFFER_DIRECTIONAL_LIGHT(b1);
CBUFFER_POINT_LIGHT(b1);
CBUFFER_IMAGE_LIGHT(b1);

Texture2D albedo : register(t0);
Texture2D world_position : register(t1);
Texture2D world_normal : register(t2);
Texture2D light_image: register(t3);

LightingInput vertex_screen(VsInput input) {
    LightingInput output;
    output.position = float4(input.position, 1.0);
    output.uv = input.uv;
    return output;
}

// Directional Light
LightingOutput pixel_directional(LightingInput input) {
    float3 intensity = directional_intensity.rgb;
    float3 direction = normalize(directional_direction.xyz);

    float3 position = world_position.Sample(globalSampler, input.uv).xyz;
    float3 normal = world_normal.Sample(globalSampler, input.uv).xyz;

    float3 camera_position = transpose(view_inv)[3].xyz;
    float diffuse_luminance = max(0, dot(normal, -direction));
    float3 reflection = normalize(direction + 2.0 * normal);
    float3 camera_ray = normalize(position - camera_position);
    float specular_intensity = max(0, pow(max(0, dot(-reflection, camera_ray)), 20.0));

    float3 diffuse_color = albedo.Sample(globalSampler, input.uv).rgb * intensity * diffuse_luminance;
    float3 specular_color = float3(specular_intensity, specular_intensity, specular_intensity);

    LightingOutput output;
    output.intensity = float4(diffuse_color + specular_color, 1.0);

    return output;
}

// Point Light
LightingOutput pixel_point(LightingInput input) {
    float3 position = world_position.Sample(globalSampler, input.uv).xyz;
    float3 normal = world_normal.Sample(globalSampler, input.uv).xyz;

    float3 light_ray = position - point_position.xyz;
    float light_distance = length(light_ray);
    float attenuation = 1.0 / (pow(light_distance + 1.0, 2.0));

    float3 light_direction = normalize(light_ray);
    float diffuse_luminance = max(0, dot(normal, -light_direction));
    float3 diffuse_color = albedo.Sample(globalSampler, input.uv).rgb * point_intensity.rgb * diffuse_luminance * attenuation;

    float3 camera_position = transpose(view_inv)[3].xyz;
    float3 reflection = normalize(light_direction + 2.0 * normal);
    float3 camera_ray = normalize(position - camera_position);
    float specular_intensity = max(0, pow(max(0, dot(-reflection, camera_ray)), 20.0));
    float3 specular_color = float3(specular_intensity, specular_intensity, specular_intensity);

    LightingOutput output;
    output.intensity = float4(diffuse_color + specular_color, 1.0);

    return output;
}

/// Image Light
LightingOutput pixel_image(LightingInput input) {
    float3 position = world_position.Sample(globalSampler, input.uv).xyz;
    float3 normal = world_normal.Sample(globalSampler, input.uv).xyz;

    float3 camera_position = transpose(view_inv)[3].xyz;
    float3 camera_ray = normalize(position - camera_position);
    float3 reflection = reflect(camera_ray, normal);

    float xz_angle = atan2(reflection.z, reflection.x);
    float yz_angle = -sign(reflection.y) * acos(dot(reflection, normalize(float3(reflection.x, 0.0, reflection.z))));
    float2 image_uv = float2((xz_angle + PI) / (2.0 * PI), yz_angle / PI + 0.5);

    LightingOutput output;
    output.intensity = float4(albedo.Sample(globalSampler, input.uv).rgb * light_image.Sample(globalSampler, image_uv).rgb * image_intensity.r, 1.0);

    return output;
}
