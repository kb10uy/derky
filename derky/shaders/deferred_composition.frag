#version 450

uniform sampler2D tex_unlit;
uniform sampler2D tex_lighting;

smooth in vec2 v_uv;

out vec4 color;

void main() {
    vec2 uv_inv = vec2(v_uv.x, 1.0 - v_uv.y);
    vec3 out_color = texture(tex_unlit, uv_inv).rgb * texture(tex_lighting, v_uv).rgb;

    // Normal output
    color = vec4(out_color, 1.0);

    // FXAA luminance
    // color = vec4(out_color.g * (0.587 / 0.299) + out_color.r);

    // Unlit
    // color = vec4(texture(tex_unlit, uv_inv).rgb, 1.0);
}
