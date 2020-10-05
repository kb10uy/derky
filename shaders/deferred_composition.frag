#version 450

uniform sampler2D tex_unlit;
// uniform sampler2D tex_lighting;

smooth in vec2 v_uv;

out vec4 color;

void main() {
    color = vec4(texture(tex_unlit, v_uv).rgb, 1.0);
}
