// ライティングパスのフラグメントシェーダー
// ディレクショナルライトを処理
// Lambert 反射

#version 450

uniform sampler2D g_position;
uniform sampler2D g_normal;
uniform vec3 lit_dir_direction;
uniform vec3 lit_dir_color;
uniform vec3 cam_position;

smooth in vec2 v_uv;

out vec4 color;

void main() {
    if (texture(g_position, v_uv).w == 0) {
        discard;
    }

    vec3 position = texture(g_position, v_uv).xyz;
    vec3 normal = texture(g_normal, v_uv).xyz;

    float diffuse_luminance = max(0, dot(normal, -lit_dir_direction));
    vec3 reflection = normalize(lit_dir_direction + 2.0 * normal);
    vec3 camera_ray = normalize(position - cam_position);
    float specular_intensity = max(0, pow(dot(-reflection, camera_ray), 20.0));
    vec3 specular_color = vec3(specular_intensity, specular_intensity, specular_intensity);

    color = vec4((lit_dir_color * diffuse_luminance) + specular_color, 1.0);
}
