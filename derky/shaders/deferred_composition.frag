#version 450

uniform sampler2D tex_unlit;
uniform sampler2D tex_lighting;

smooth in vec2 v_uv;

out vec4 color;

void main() {
    vec2 uv_inv = vec2(v_uv.x, 1.0 - v_uv.y);
    vec3 out_color = texture(tex_unlit, uv_inv).rgb * texture(tex_lighting, v_uv).rgb;
    color = vec4(out_color, 1.0);
    // color = vec4(out_color.g * (0.587 / 0.299) + out_color.r);
}
