#version 450

const float PI = 3.14159265;

uniform sampler2D g_position;
uniform sampler2D g_normal;
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

    // XZ 平面上の角
    // XZ 平面の射影となす角
    // yz_angle の範囲は [0, π/2] になるので normal.y の符号をかける
    float xz_angle = atan(normal.z, normal.x);
    float yz_angle = sign(normal.y) * acos(dot(normal, normalize(vec3(normal.x, 0.0, normal.z))));
    vec2 image_uv = vec2((xz_angle + PI) / 2.0, yz_angle / PI + 0.5);

    color = vec4(texture(light_image_source, image_uv).xyz * light_image_intensity, 1.0);
}
