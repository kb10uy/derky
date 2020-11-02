#version 450

uniform sampler2D tex_unlit;
uniform sampler2D tex_lighting;

smooth in vec2 v_uv;

out vec4 color;

void main() {
    vec2 uv_inv = vec2(v_uv.x, 1.0 - v_uv.y);
    vec3 unlit_color = texture(tex_unlit, uv_inv).rgb;
    vec3 light_color = texture(tex_lighting, v_uv).rgb;

    vec3 out_color = unlit_color * light_color * 1.0;

    /*
    // Exposure Visualization
    if (out_color.r > 1.0 || out_color.g > 1.0 || out_color.b > 1.0) {
        color = vec4(0.0, 1.0, 0.0, 1.0);
    } else if (light_color.r < 0.2 && light_color.g < 0.2 && light_color.b < 0.2) {
        color = vec4(0.0, 0.0, 1.0, 1.0);
    } else {
        color = vec4(out_color, 1.0);
    }
    */


    // Normal output
    color = vec4(out_color, 1.0);

    // FXAA luminance
    // color = vec4(out_color.g * (0.587 / 0.299) + out_color.r);

    // Unlit
    // color = vec4(texture(tex_unlit, uv_inv).rgb, 1.0);
}
