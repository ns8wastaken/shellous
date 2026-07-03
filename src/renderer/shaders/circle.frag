#version 300 es
precision highp float;

uniform vec2 u_rect_size; // Bounding box dimensions
uniform vec4 u_color;
uniform vec4 u_border_color;
uniform int u_fill_mode;
uniform vec2 u_gradient_direction;
uniform vec4 u_gradient_stops;
uniform vec4 u_gradient_color0;
uniform vec4 u_gradient_color1;
uniform vec4 u_gradient_color2;
uniform vec4 u_gradient_color3;

uniform float u_softness;
uniform int u_no_aa;
uniform int u_invert_fill;
uniform float u_border_width;
uniform int u_outer_shadow;
uniform vec2 u_shadow_cutout_offset;

in vec2 v_pixel;
out vec4 fragColor;

// Returns (signed distance, is_corner).
// For a perfect circle, we use the minimum radius of the bounding box.
vec2 circle_distance_with_corner(vec2 point, vec2 size) {
    vec2 center = size * 0.5;
    float radius = min(size.x, size.y) * 0.5;
    float dist = length(point - center) - radius;
    // A circle is entirely curved, so is_corner is always 1.0 to trigger aa_curve smoothing
    return vec2(dist, 1.0);
}

float circle_distance(vec2 point, vec2 size) {
    return circle_distance_with_corner(point, size).x;
}

// Pixel-grid-snap window for anti-aliasing
float coverage_for(vec2 distance_with_corner, float aa_curve) {
    if (u_no_aa == 1) {
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
    vec4 stops = clamp(u_gradient_stops, vec4(0.0), vec4(1.0));
    stops.y = max(stops.y, stops.x);
    stops.z = max(stops.z, stops.y);
    stops.w = max(stops.w, stops.z);

    vec4 c0 = u_gradient_color0;
    vec4 c1 = u_gradient_color1;
    vec4 c2 = u_gradient_color2;
    vec4 c3 = u_gradient_color3;

    if (position <= stops.y) {
        return mix(c0, c1, gradient_segment_t(position, stops.x, stops.y));
    }
    if (position <= stops.z) {
        return mix(c1, c2, gradient_segment_t(position, stops.y, stops.z));
    }
    return mix(c2, c3, gradient_segment_t(position, stops.z, stops.w));
}

void main() {
    float aa = max(u_softness, 0.85);
    vec2 local_point = v_pixel;
    vec2 uv = clamp(local_point / u_rect_size, vec2(0.0), vec2(1.0));

    vec2 outer = circle_distance_with_corner(local_point, u_rect_size);
    float outer_distance = outer.x;
    float outer_coverage = coverage_for(outer, aa);

    if (u_invert_fill == 1) outer_coverage = 1.0 - outer_coverage;

    // Handle Outer Shadows
    if (u_outer_shadow == 1) {
        float cutout_aa = 0.85;
        float shadow_distance = circle_distance(local_point, u_rect_size);
        float shadow_outer_coverage = 1.0 - smoothstep(-aa, aa, shadow_distance);

        float cutout_distance = circle_distance(local_point + u_shadow_cutout_offset, u_rect_size);
        float cutout_mask = 1.0 - smoothstep(-cutout_aa, cutout_aa, cutout_distance);

        float shadow_coverage = shadow_outer_coverage * (1.0 - cutout_mask);
        float out_alpha = u_color.a * shadow_coverage;

        if (out_alpha <= 0.0) {
            discard;
        }
        fragColor = vec4(u_color.rgb * out_alpha, out_alpha);
        return;
    }

    // Handle Fills
    float gradient_pos = clamp(dot(uv, u_gradient_direction), 0.0, 1.0);
    vec4 fill_base;
    if (u_fill_mode == 0) {
        fill_base = vec4(0.0);
    } else if (u_fill_mode == 1) {
        fill_base = u_color;
    } else {
        fill_base = gradient_fill(gradient_pos);
    }

    // Fast-path: No Border
    if (u_border_width <= 0.0 || u_border_color.a <= 0.0) {
        float out_alpha = fill_base.a * outer_coverage;
        if (out_alpha <= 0.0) {
            discard;
        }
        fragColor = vec4(fill_base.rgb * out_alpha, out_alpha);
        return;
    }

    // Calculate Inner Circle for Border Masking
    vec2 inner_size = max(u_rect_size - vec2(u_border_width * 2.0), vec2(0.0));
    vec2 inner_point = local_point - vec2(u_border_width);
    vec2 inner = circle_distance_with_corner(inner_point, inner_size);
    float inner_coverage = coverage_for(inner, aa);

    // Border-only (Hollow Fill)
    if (fill_base.a <= 0.0) {
        float ring_coverage = outer_coverage * (1.0 - inner_coverage);
        float out_alpha = u_border_color.a * ring_coverage;
        if (out_alpha <= 0.0) {
            discard;
        }
        fragColor = vec4(u_border_color.rgb * out_alpha, out_alpha);
        return;
    }

    // Mix Border and Fill smoothly
    vec3 border_pm = u_border_color.rgb * u_border_color.a;
    vec3 fill_pm = fill_base.rgb * fill_base.a;

    vec3 interior_rgb = mix(border_pm, fill_pm, inner_coverage);
    float interior_a = mix(u_border_color.a, fill_base.a, inner_coverage);

    float out_alpha = interior_a * outer_coverage;
    if (out_alpha <= 0.0) {
        discard;
    }

    fragColor = vec4(interior_rgb * outer_coverage, out_alpha);
}
