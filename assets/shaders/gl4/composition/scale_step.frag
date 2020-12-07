#version 460

uniform sampler2D scale_prev_texture;
uniform vec2 scale_prev_size;

in vec2 v_uv;

out vec4 color;

void main() {
    vec2 uv_inv = vec2(v_uv.x, 1.0 - v_uv.y);
    vec2 uv_pixel = vec2(1.0, 1.0) / 1024.0;
    vec4 result = 0.0
        + texture(scale_prev_texture, vec2(uv_inv.x - uv_pixel.x * 1.5, uv_inv.y - uv_pixel.y * 1.5))
        + texture(scale_prev_texture, vec2(uv_inv.x - uv_pixel.x * 0.5, uv_inv.y - uv_pixel.y * 1.5))
        + texture(scale_prev_texture, vec2(uv_inv.x + uv_pixel.x * 0.5, uv_inv.y - uv_pixel.y * 1.5))
        + texture(scale_prev_texture, vec2(uv_inv.x + uv_pixel.x * 1.5, uv_inv.y - uv_pixel.y * 1.5))
        + texture(scale_prev_texture, vec2(uv_inv.x - uv_pixel.x * 1.5, uv_inv.y - uv_pixel.y * 0.5))
        + texture(scale_prev_texture, vec2(uv_inv.x - uv_pixel.x * 0.5, uv_inv.y - uv_pixel.y * 0.5))
        + texture(scale_prev_texture, vec2(uv_inv.x + uv_pixel.x * 0.5, uv_inv.y - uv_pixel.y * 0.5))
        + texture(scale_prev_texture, vec2(uv_inv.x + uv_pixel.x * 1.5, uv_inv.y - uv_pixel.y * 0.5))
        + texture(scale_prev_texture, vec2(uv_inv.x - uv_pixel.x * 1.5, uv_inv.y + uv_pixel.y * 0.5))
        + texture(scale_prev_texture, vec2(uv_inv.x - uv_pixel.x * 0.5, uv_inv.y + uv_pixel.y * 0.5))
        + texture(scale_prev_texture, vec2(uv_inv.x + uv_pixel.x * 0.5, uv_inv.y + uv_pixel.y * 0.5))
        + texture(scale_prev_texture, vec2(uv_inv.x + uv_pixel.x * 1.5, uv_inv.y + uv_pixel.y * 0.5))
        + texture(scale_prev_texture, vec2(uv_inv.x - uv_pixel.x * 1.5, uv_inv.y + uv_pixel.y * 1.5))
        + texture(scale_prev_texture, vec2(uv_inv.x - uv_pixel.x * 0.5, uv_inv.y + uv_pixel.y * 1.5))
        + texture(scale_prev_texture, vec2(uv_inv.x + uv_pixel.x * 0.5, uv_inv.y + uv_pixel.y * 1.5))
        + texture(scale_prev_texture, vec2(uv_inv.x + uv_pixel.x * 1.5, uv_inv.y + uv_pixel.y * 1.5));

    result /= 16.0;
    color = vec4(result.rgb, 1.0);
}
