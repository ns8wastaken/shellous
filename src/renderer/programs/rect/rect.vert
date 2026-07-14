#version 300 es
precision highp float;

layout(location = 0) in vec2 a_position;
layout(location = 1) in vec4 a_inst0;
layout(location = 2) in vec4 a_inst1;
layout(location = 3) in vec4 a_inst2;
layout(location = 4) in vec4 a_inst3;
layout(location = 5) in vec4 a_inst4;
layout(location = 6) in vec4 a_inst5;
layout(location = 7) in vec4 a_inst6;
layout(location = 8) in vec4 a_inst7;
layout(location = 9) in vec4 a_inst8;
layout(location = 10) in vec4 a_inst9;
layout(location = 11) in vec4 a_inst10;
layout(location = 12) in vec4 a_inst11;
layout(location = 13) in vec4 a_inst12;
layout(location = 14) in vec4 a_inst13;
layout(location = 15) in vec4 a_inst14;

uniform vec2 u_surface_size;

out vec2 v_pixel;
out vec2 v_rect_size;
out vec4 v_color;
out vec4 v_border_color;
out float v_fill_mode;
out vec2 v_gradient_direction;
out vec4 v_gradient_stops;
out vec4 v_gradient_color0;
out vec4 v_gradient_color1;
out vec4 v_gradient_color2;
out vec4 v_gradient_color3;
out vec4 v_corner_shapes;
out vec4 v_logical_inset;
out vec4 v_radii;
out float v_softness;
out float v_no_aa;
out float v_invert_fill;
out float v_border_width;
out float v_outer_shadow;
out vec2 v_shadow_cutout_offset;

vec2 to_ndc(vec2 pixel_pos) {
    vec2 normalized = pixel_pos / u_surface_size;
    return vec2(normalized.x * 2.0 - 1.0, 1.0 - normalized.y * 2.0);
}

void main() {
    vec2 local = a_position * a_inst0.zw;
    vec2 pixel = local + a_inst0.xy;
    v_pixel = local - vec2(a_inst1.x);
    v_rect_size = a_inst1.yz;
    v_fill_mode = a_inst1.w;
    v_color = a_inst2;
    v_border_color = a_inst3;
    v_gradient_direction = a_inst4.xy;
    v_softness = a_inst4.z;
    v_no_aa = a_inst4.w;
    v_invert_fill = a_inst5.x;
    v_border_width = a_inst5.y;
    v_outer_shadow = a_inst5.z;
    v_shadow_cutout_offset = vec2(a_inst5.w, a_inst6.x);
    v_gradient_stops = a_inst7;
    v_gradient_color0 = a_inst8;
    v_gradient_color1 = a_inst9;
    v_gradient_color2 = a_inst10;
    v_gradient_color3 = a_inst11;
    v_corner_shapes = a_inst12;
    v_logical_inset = a_inst13;
    v_radii = a_inst14;
    gl_Position = vec4(to_ndc(pixel), 0.0, 1.0);
}