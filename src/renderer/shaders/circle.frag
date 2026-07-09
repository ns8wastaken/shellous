#version 300 es
precision highp float;

in vec2 v_pixel;
in vec2 v_rect_size;
in vec4 v_color;
in vec4 v_border_color;
in float v_fill_mode;
in vec2 v_gradient_direction;
in vec4 v_gradient_stops;
in vec4 v_gradient_color0;
in vec4 v_gradient_color1;
in vec4 v_gradient_color2;
in vec4 v_gradient_color3;
in float v_softness;
in float v_no_aa;
in float v_invert_fill;
in float v_border_width;
in float v_outer_shadow;
in vec2 v_shadow_cutout_offset;

out vec4 fragColor;

vec2 circle_distance_with_corner(vec2 point, vec2 size) {
    vec2 center = size * 0.5;
    float radius = min(size.x, size.y) * 0.5;
    float dist = length(point - center) - radius;
    return vec2(dist, 1.0);
}

float circle_distance(vec2 point, vec2 size) {
    return circle_distance_with_corner(point, size).x;
}

float coverage_for(vec2 distance_with_corner, float aa_curve) {
    if (v_no_aa > 0.5) {
        return 1.0 - step(0.0, distance_with_corner.x);
    }
    float lo = mix(-0.5, -aa_curve, distance_with_corner.y);
    float hi = mix( 0.5,  aa_curve, distance_with_corner.y);
    return 1.0 - smoothstep(lo, hi, distance_with_corner.x);
}

float gradient_segment_t(float position, float start, float end) {
    return clamp((position - start) / max(end - start, 0.0001), 0.0, 1.0);
}

vec4 gradient_fill(float position) {
    vec4 stops = clamp(v_gradient_stops, vec4(0.0), vec4(1.0));
    stops.y = max(stops.y, stops.x);
    stops.z = max(stops.z, stops.y);
    stops.w = max(stops.w, stops.z);

    vec4 c0 = v_gradient_color0;
    vec4 c1 = v_gradient_color1;
    vec4 c2 = v_gradient_color2;
    vec4 c3 = v_gradient_color3;

    if (position <= stops.y) {
        return mix(c0, c1, gradient_segment_t(position, stops.x, stops.y));
    }
    if (position <= stops.z) {
        return mix(c1, c2, gradient_segment_t(position, stops.y, stops.z));
    }
    return mix(c2, c3, gradient_segment_t(position, stops.z, stops.w));
}

void main() {
    float aa = max(v_softness, 0.85);
    vec2 local_point = v_pixel;
    vec2 uv = clamp(local_point / v_rect_size, vec2(0.0), vec2(1.0));

    vec2 outer = circle_distance_with_corner(local_point, v_rect_size);
    float outer_distance = outer.x;
    float outer_coverage = coverage_for(outer, aa);

    if (v_invert_fill > 0.5) outer_coverage = 1.0 - outer_coverage;

    if (v_outer_shadow > 0.5) {
        float cutout_aa = 0.85;
        float shadow_distance = circle_distance(local_point, v_rect_size);
        float shadow_outer_coverage = 1.0 - smoothstep(-aa, aa, shadow_distance);

        float cutout_distance = circle_distance(local_point + v_shadow_cutout_offset, v_rect_size);
        float cutout_mask = 1.0 - smoothstep(-cutout_aa, cutout_aa, cutout_distance);

        float shadow_coverage = shadow_outer_coverage * (1.0 - cutout_mask);
        float out_alpha = v_color.a * shadow_coverage;

        if (out_alpha <= 0.0) {
            discard;
        }
        fragColor = vec4(v_color.rgb * out_alpha, out_alpha);
        return;
    }

    float gradient_pos = clamp(dot(uv, v_gradient_direction), 0.0, 1.0);
    vec4 fill_base;
    if (v_fill_mode < 0.5) {
        fill_base = vec4(0.0);
    } else if (v_fill_mode < 1.5) {
        fill_base = v_color;
    } else {
        fill_base = gradient_fill(gradient_pos);
    }

    if (v_border_width <= 0.0 || v_border_color.a <= 0.0) {
        float out_alpha = fill_base.a * outer_coverage;
        if (out_alpha <= 0.0) {
            discard;
        }
        fragColor = vec4(fill_base.rgb * out_alpha, out_alpha);
        return;
    }

    vec2 inner_size = max(v_rect_size - vec2(v_border_width * 2.0), vec2(0.0));
    vec2 inner_point = local_point - vec2(v_border_width);
    vec2 inner = circle_distance_with_corner(inner_point, inner_size);
    float inner_coverage = coverage_for(inner, aa);

    if (fill_base.a <= 0.0) {
        float ring_coverage = outer_coverage * (1.0 - inner_coverage);
        float out_alpha = v_border_color.a * ring_coverage;
        if (out_alpha <= 0.0) {
            discard;
        }
        fragColor = vec4(v_border_color.rgb * out_alpha, out_alpha);
        return;
    }

    vec3 border_pm = v_border_color.rgb * v_border_color.a;
    vec3 fill_pm = fill_base.rgb * fill_base.a;

    vec3 interior_rgb = mix(border_pm, fill_pm, inner_coverage);
    float interior_a = mix(v_border_color.a, fill_base.a, inner_coverage);

    float out_alpha = interior_a * outer_coverage;
    if (out_alpha <= 0.0) {
        discard;
    }

    fragColor = vec4(interior_rgb * outer_coverage, out_alpha);
}