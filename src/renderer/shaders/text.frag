#version 330 core

in vec2 v_uv;

uniform sampler2D u_atlas;
uniform vec4 u_text_color;

out vec4 f_color;

void main() {
    float alpha = texture(u_atlas, v_uv).r;
    f_color = vec4(u_text_color.rgb, u_text_color.a * alpha);
}
