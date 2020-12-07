#version 450

const float PI = 3.14159265;

uniform sampler2D g_position;
uniform sampler2D g_normal;
uniform vec3 env_camera_position;
uniform sampler2D light_image_source;
uniform float light_image_intensity;

in vec2 v_uv;

out vec4 color;

void main() {
    if (texture(g_position, v_uv).w == 0) {
        discard;
    }

    vec3 position = texture(g_position, v_uv).xyz;
    vec3 normal = texture(g_normal, v_uv).xyz;

    vec3 camera_ray = normalize(position - env_camera_position);
    vec3 reflection = camera_ray - 2.0 * normal * dot(normal, camera_ray);

    // XZ 平面上の角
    // XZ 平面の射影となす角
    // yz_angle の範囲は [0, π/2] になるので reflection.y の符号をかける
    float xz_angle = atan(reflection.z, reflection.x);
    float yz_angle = sign(reflection.y) * acos(dot(reflection, normalize(vec3(reflection.x, 0.0, reflection.z))));
    vec2 image_uv = vec2((xz_angle + PI) / (2.0 * PI), yz_angle / PI + 0.5);

    color = vec4(texture(light_image_source, vec2(image_uv.x, 1.0 - image_uv.y)).xyz * light_image_intensity, 1.0);
}
