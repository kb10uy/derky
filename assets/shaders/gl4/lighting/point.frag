#version 450

uniform sampler2D g_position;
uniform sampler2D g_normal;
uniform vec3 light_point_intensity;
uniform vec3 light_point_position;

smooth in vec2 v_uv;

out vec4 color;

void main() {
    vec3 position = texture(g_position, v_uv).xyz;
    vec3 normal = texture(g_normal, v_uv).xyz;

    // TODO: きめうちにしない
    vec3 light_ray = position - light_point_position;
    vec3 light_direction = normalize(light_ray);
    float light_distance = length(light_ray);
    float attenuation = 1.0 / (pow(light_distance + 1.0, 2.0));

    float diffuse_luminance = max(0, dot(normal, -light_direction));
    vec3 out_color = light_point_intensity * attenuation * diffuse_luminance;

    color = vec4(out_color, 1.0);
}
