// ライティングパスのバーテックスシェーダー
// 基本的にスクリーン全体を覆うだけ

#version 450

uniform mat4 env_screen_matrix;

in vec4 position;
in vec2 uv;

smooth out vec2 v_uv;

void main() {
    v_uv = uv;
    gl_Position = env_screen_matrix * position;
}
