#version 300 es
precision highp float;

in vec2 v_uv;
uniform sampler2D u_atlas;
uniform vec4 u_text_color;
out vec4 f_color;

void main() {
    float mask = texture(u_atlas, v_uv).r;
    float final_alpha = u_text_color.a * mask;

    f_color = vec4(u_text_color.rgb * final_alpha, final_alpha);
}
