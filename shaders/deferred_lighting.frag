// ライティングパスのフラグメントシェーダー
// ディレクショナルライトを処理
// Lambert 反射

#version 450

uniform sampler2D g_position;
uniform sampler2D g_normal;
uniform vec3 lit_dir_direction;
uniform vec3 lit_dir_color;

smooth in vec2 v_uv;

out vec4 color;

void main() {
    float diffuse_luminance = dot(texture(g_normal, v_uv).xyz, -lit_dir_direction);
    color = vec4(lit_dir_color * diffuse_luminance, 1.0);
}
