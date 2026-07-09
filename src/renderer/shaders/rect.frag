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
in vec4 v_corner_shapes;
in vec4 v_logical_inset;
in vec4 v_radii;
in float v_softness;
in float v_no_aa;
in float v_invert_fill;
in float v_border_width;
in float v_outer_shadow;
in vec2 v_shadow_cutout_offset;
out vec4 fragColor;

vec2 rounded_rect_distance_with_corner(vec2 point, vec2 size, vec4 radii) {
    vec2 half_size = size * 0.5;
    vec2 centered = point - half_size;
    float r = centered.x < 0.0
        ? (centered.y < 0.0 ? radii.x : radii.w)
        : (centered.y < 0.0 ? radii.y : radii.z);
    vec2 q = abs(centered) - (half_size - vec2(r));
    float distance = length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
    float is_corner = (r > 0.0 && q.x > 0.0 && q.y > 0.0) ? 1.0 : 0.0;
    return vec2(distance, is_corner);
}

float rounded_rect_distance(vec2 point, vec2 size, vec4 radii) {
    return rounded_rect_distance_with_corner(point, size, radii).x;
}

float circle_extent(float radius, float delta) {
    return sqrt(max(0.0, radius * radius - delta * delta));
}

vec2 shape_distance_with_corner(vec2 point, vec2 size, vec4 radii, vec4 corner_shapes, vec4 logical_inset) {
    vec4 safe_inset = max(logical_inset, vec4(0.0));
    vec2 body_min = min(safe_inset.xy, size);
    vec2 body_max = max(body_min, size - safe_inset.zw);
    vec2 body_size = max(body_max - body_min, vec2(0.0));
    vec4 r = max(radii, vec4(0.0));

    bool tl_concave = corner_shapes.x > 0.5;
    bool tr_concave = corner_shapes.y > 0.5;
    bool br_concave = corner_shapes.z > 0.5;
    bool bl_concave = corner_shapes.w > 0.5;
    bool any_concave = tl_concave || tr_concave || br_concave || bl_concave;

    if (!any_concave) {
        return rounded_rect_distance_with_corner(point - body_min, body_size, r);
    }

    bool in_corner_box = false;
    if (r.x > 0.0 && point.x < body_min.x + r.x && point.y < body_min.y + r.x) in_corner_box = true;
    if (r.y > 0.0 && point.x > body_max.x - r.y && point.y < body_min.y + r.y) in_corner_box = true;
    if (r.z > 0.0 && point.x > body_max.x - r.z && point.y > body_max.y - r.z) in_corner_box = true;
    if (r.w > 0.0 && point.x < body_min.x + r.w && point.y > body_max.y - r.w) in_corner_box = true;

    float x = point.x;
    float y = point.y;
    float left = body_min.x;
    float right = body_max.x;
    float top = body_min.y;
    float bottom = body_max.y;

    float radius = r.x;
    if (radius > 0.0 && y < body_min.y + radius) {
        float sample_y = clamp(y, body_min.y, body_min.y + radius);
        float dy = sample_y - (body_min.y + radius);
        float extent = circle_extent(radius, dy);
        if (tl_concave) {
            left = min(left, body_min.x - radius + extent);
        } else {
            left = max(left, body_min.x + radius - extent);
        }
    }
    if (radius > 0.0 && x < body_min.x + radius) {
        float sample_x = clamp(x, body_min.x, body_min.x + radius);
        float dx = sample_x - (body_min.x + radius);
        float extent = circle_extent(radius, dx);
        if (tl_concave) {
            top = min(top, body_min.y - radius + extent);
        } else {
            top = max(top, body_min.y + radius - extent);
        }
    }

    radius = r.y;
    if (radius > 0.0 && y < body_min.y + radius) {
        float sample_y = clamp(y, body_min.y, body_min.y + radius);
        float dy = sample_y - (body_min.y + radius);
        float extent = circle_extent(radius, dy);
        if (tr_concave) {
            right = max(right, body_max.x + radius - extent);
        } else {
            right = min(right, body_max.x - radius + extent);
        }
    }
    if (radius > 0.0 && x > body_max.x - radius) {
        float sample_x = clamp(x, body_max.x - radius, body_max.x);
        float dx = sample_x - (body_max.x - radius);
        float extent = circle_extent(radius, dx);
        if (tr_concave) {
            top = min(top, body_min.y - radius + extent);
        } else {
            top = max(top, body_min.y + radius - extent);
        }
    }

    radius = r.z;
    if (radius > 0.0 && y > body_max.y - radius) {
        float sample_y = clamp(y, body_max.y - radius, body_max.y);
        float dy = sample_y - (body_max.y - radius);
        float extent = circle_extent(radius, dy);
        if (br_concave) {
            right = max(right, body_max.x + radius - extent);
        } else {
            right = min(right, body_max.x - radius + extent);
        }
    }
    if (radius > 0.0 && x > body_max.x - radius) {
        float sample_x = clamp(x, body_max.x - radius, body_max.x);
        float dx = sample_x - (body_max.x - radius);
        float extent = circle_extent(radius, dx);
        if (br_concave) {
            bottom = max(bottom, body_max.y + radius - extent);
        } else {
            bottom = min(bottom, body_max.y - radius + extent);
        }
    }

    radius = r.w;
    if (radius > 0.0 && y > body_max.y - radius) {
        float sample_y = clamp(y, body_max.y - radius, body_max.y);
        float dy = sample_y - (body_max.y - radius);
        float extent = circle_extent(radius, dy);
        if (bl_concave) {
            left = min(left, body_min.x - radius + extent);
        } else {
            left = max(left, body_min.x + radius - extent);
        }
    }
    if (radius > 0.0 && x < body_min.x + radius) {
        float sample_x = clamp(x, body_min.x, body_min.x + radius);
        float dx = sample_x - (body_min.x + radius);
        float extent = circle_extent(radius, dx);
        if (bl_concave) {
            bottom = max(bottom, body_max.y + radius - extent);
        } else {
            bottom = min(bottom, body_max.y - radius + extent);
        }
    }

    float boundary_distance = max(max(left - x, x - right), max(top - y, y - bottom));
    float visual_clip = max(max(-point.x, point.x - size.x), max(-point.y, point.y - size.y));
    float is_corner = (in_corner_box && boundary_distance > visual_clip) ? 1.0 : 0.0;
    return vec2(max(boundary_distance, visual_clip), is_corner);
}

float shape_distance(vec2 point, vec2 size, vec4 radii, vec4 corner_shapes, vec4 logical_inset) {
    return shape_distance_with_corner(point, size, radii, corner_shapes, logical_inset).x;
}

float shadow_shape_distance(vec2 point, vec2 size, vec4 radii, vec4 corner_shapes, vec4 logical_inset) {
    float distance = shape_distance(point, size, radii, corner_shapes, logical_inset);

    vec4 safe_inset = max(logical_inset, vec4(0.0));
    vec2 body_min = min(safe_inset.xy, size);
    vec2 body_max = max(body_min, size - safe_inset.zw);
    vec2 body_size = max(body_max - body_min, vec2(0.0));
    vec4 r = max(radii, vec4(0.0));

    bool tl_concave = corner_shapes.x > 0.5;
    bool tr_concave = corner_shapes.y > 0.5;
    bool br_concave = corner_shapes.z > 0.5;
    bool bl_concave = corner_shapes.w > 0.5;
    bool any_concave = tl_concave || tr_concave || br_concave || bl_concave;
    if (!any_concave) {
        return distance;
    }

    float x = point.x;
    float y = point.y;
    float left_distance = body_min.x - x;
    float right_distance = x - body_max.x;
    float top_distance = body_min.y - y;
    float bottom_distance = y - body_max.y;

    float radius = r.x;
    if (!tl_concave && radius > 0.0 && x < body_min.x + radius && y < body_min.y + radius) {
        float corner_distance = length(point - vec2(body_min.x + radius, body_min.y + radius)) - radius;
        distance = max(max(right_distance, bottom_distance), corner_distance);
    }

    radius = r.y;
    if (!tr_concave && radius > 0.0 && x > body_max.x - radius && y < body_min.y + radius) {
        float corner_distance = length(point - vec2(body_max.x - radius, body_min.y + radius)) - radius;
        distance = max(max(left_distance, bottom_distance), corner_distance);
    }

    radius = r.z;
    if (!br_concave && radius > 0.0 && x > body_max.x - radius && y > body_max.y - radius) {
        float corner_distance = length(point - vec2(body_max.x - radius, body_max.y - radius)) - radius;
        distance = max(max(left_distance, top_distance), corner_distance);
    }

    radius = r.w;
    if (!bl_concave && radius > 0.0 && x < body_min.x + radius && y > body_max.y - radius) {
        float corner_distance = length(point - vec2(body_min.x + radius, body_max.y - radius)) - radius;
        distance = max(max(right_distance, top_distance), corner_distance);
    }

    float visual_clip = max(max(-point.x, point.x - size.x), max(-point.y, point.y - size.y));
    return max(distance, visual_clip);
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

    vec2 outer = shape_distance_with_corner(local_point, v_rect_size, v_radii, v_corner_shapes, v_logical_inset);
    float outer_distance = outer.x;
    float outer_coverage = coverage_for(outer, aa);
    if (v_invert_fill > 0.5) outer_coverage = 1.0 - outer_coverage;

    if (v_outer_shadow > 0.5) {
        float cutout_aa = 0.85;
        float shadow_distance = shadow_shape_distance(local_point, v_rect_size, v_radii, v_corner_shapes, v_logical_inset);
        float shadow_outer_coverage = 1.0 - smoothstep(-aa, aa, shadow_distance);
        float cutout_distance = shape_distance(local_point + v_shadow_cutout_offset, v_rect_size, v_radii, v_corner_shapes, v_logical_inset);
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

    bool any_concave = v_corner_shapes.x > 0.5 || v_corner_shapes.y > 0.5 || v_corner_shapes.z > 0.5 || v_corner_shapes.w > 0.5;
    vec2 inner;
    if (any_concave) {
        inner = vec2(outer_distance + v_border_width, outer.y);
    } else {
        vec4 inner_radii = max(v_radii - vec4(v_border_width), vec4(0.0));
        vec2 inner_size = max(v_rect_size - vec2(v_border_width * 2.0), vec2(0.0));
        vec2 inner_point = local_point - vec2(v_border_width);
        vec4 inner_inset = max(v_logical_inset - vec4(v_border_width), vec4(0.0));
        inner = shape_distance_with_corner(inner_point, inner_size, inner_radii, v_corner_shapes, inner_inset);
    }
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