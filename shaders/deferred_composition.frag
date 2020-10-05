// 合成用のフラグメントシェーダー
// unlit 状態のバッファとライティング結果を合成する

#version 450

uniform sampler2D tex_unlit;
uniform sampler2D tex_lighting;

smooth in vec2 v_uv;

out vec4 color;

void main() {
    color = vec4(texture(tex_lighting, v_uv).rgb, 1.0);
}
