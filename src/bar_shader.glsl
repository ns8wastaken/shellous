#version 330

in vec2 uv;
out vec4 color;

uniform vec2 resolution;
uniform int ws_count;
uniform int active_slot;
uniform int hover_slot;

// ---- Signed Distance Functions ----

float sdCircle(vec2 p, float r) {
    return length(p) - r;
}

float sdBox(vec2 p, vec2 b) {
    vec2 d = abs(p) - b;
    return length(max(d, 0.0)) + min(max(d.x, d.y), 0.0);
}

float sdCapsule(vec2 p, vec2 a, vec2 b, float r) {
    vec2 pa = p - a;
    vec2 ba = b - a;
    float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h) - r;
}

// ---- Main ----

void main() {
    vec2 p = uv * resolution;
    float hh = resolution.y * 0.5;

    // Start fully transparent
    vec3 col = vec3(0.0);
    float alpha = 0.0;

    // ==================== PANEL GEOMETRY ====================
    // The panel sits at the top of the screen, flush with the left edge.
    //   top    = resolution.y    (screen edge)
    //   bottom = 0               (facing the desktop)
    //   left   = 0
    //   right  = panel_w

    float panel_w = 260.0;      // panel width from left edge
    float cr = 15.0;            // convex corner radius (bottom-left)
    float concave_r = 25.0;     // concave transition radius (right edge)

    // --- Main rectangle ---
    vec2 half = vec2(panel_w * 0.5, hh);
    vec2 center = vec2(panel_w * 0.5, hh);
    float rect_sdf = sdBox(p - center, half);

    // --- Convex bottom-left corner ---
    // Circle at (0, 0) bulging outward below the panel.
    float bl_sdf = sdCircle(p - vec2(0.0, 0.0), cr);

    // Union: rectangle + convex corner
    float panel_sdf = min(rect_sdf, bl_sdf);

    // --- Concave right-edge transition ---
    // Subtract circle at (panel_w, 0) to create an inward curve
    // that smoothly transitions the right edge upward into the background.
    float concave_sdf = sdCircle(p - vec2(panel_w, 0.0), concave_r);
    panel_sdf = max(panel_sdf, -concave_sdf);

    float panel_a = 1.0 - smoothstep(0.0, 1.0, panel_sdf);

    if (panel_a > 0.0) {
        // ---- Panel background ----
        vec3 panel_bg = vec3(0.085, 0.095, 0.110);

        // Subtle vertical gradient
        panel_bg = mix(panel_bg, vec3(0.100, 0.110, 0.125), p.y / resolution.y);

        // ---- Border stroke ----
        // Thin inner stroke along the bottom edge and bottom-left curve.
        float stroke_width = 1.5;
        float stroke_sdf = abs(panel_sdf) - stroke_width;
        float stroke_a = 1.0 - smoothstep(0.0, 0.5, stroke_sdf);

        // Mask: only along the bottom portion (y near 0) and the convex corner.
        // Gaussian falloff: 1 at y=0, fading toward the top.
        float bottom_mask = exp(-(p.y * p.y) / 80.0);

        // Exclude the right side near the concave transition so the
        // stroke does not wrap into the inverted curve.
        float right_mask = 1.0 - smoothstep(panel_w - 30.0, panel_w + 2.0, p.x);
        bottom_mask *= right_mask;

        stroke_a *= bottom_mask;

        vec3 stroke_col = vec3(0.50, 0.60, 0.78);  // light blue-silver
        panel_bg = mix(panel_bg, stroke_col, stroke_a * 0.55);

        // ==================== WORKSPACE INDICATORS ====================
        // Dynamic layout from Hyprland workspace data.
        // - inactive workspaces: small circles (dots)
        // - active workspace: elongated horizontal capsule (pill)

        float start_x = 20.0;
        float spacing = 22.0;
        float dot_r = 2.5;          // inactive dot radius
        float cap_r = 3.5;          // capsule end radius
        float cap_half = 5.5;       // capsule half-length
        float elem_y = hh;          // vertically centred

        for (int i = 0; i < ws_count && i < 20; i++) {
            // Stop if this element would extend past the panel edge
            float cx = start_x + float(i) * spacing;
            if (cx + cap_half + cap_r > panel_w) break;

            vec2 c = p - vec2(cx, elem_y);

            if (i == active_slot) {
                // ---- Active: elongated capsule ----
                float d = sdCapsule(c, vec2(-cap_half, 0.0), vec2(cap_half, 0.0), cap_r);
                float a = 1.0 - smoothstep(0.0, 1.0, d);

                vec3 active_col = vec3(0.48, 0.62, 0.82);  // brighter light blue-grey

                // Inner highlight
                float inner_d = sdCapsule(
                    c, vec2(-cap_half * 0.6, 0.0), vec2(cap_half * 0.6, 0.0), cap_r * 0.6
                );
                float inner = 1.0 - smoothstep(0.0, 1.0, inner_d);
                active_col += vec3(0.10, 0.12, 0.14) * inner;

                panel_bg = mix(panel_bg, active_col, a);

                // Hover glow on capsule
                if (i == hover_slot) {
                    float glow_d = sdCapsule(
                        c, vec2(-cap_half, 0.0), vec2(cap_half, 0.0), cap_r + 3.0
                    );
                    float glow = 1.0 - smoothstep(0.0, 3.0, glow_d);
                    panel_bg += vec3(0.55, 0.70, 0.90) * glow * 0.12 * (1.0 - a);
                }

            } else {
                // ---- Inactive: small circle (dot) ----
                float d = sdCircle(c, dot_r);
                float a = 1.0 - smoothstep(0.0, 1.0, d);

                vec3 dot_col;
                if (i == hover_slot) {
                    dot_col = vec3(0.35, 0.40, 0.50);  // brighter on hover
                    float glow_d = sdCircle(c, dot_r + 3.0);
                    float glow = 1.0 - smoothstep(0.0, 3.0, glow_d);
                    panel_bg += vec3(0.40, 0.50, 0.65) * glow * 0.10 * (1.0 - a);
                } else {
                    dot_col = vec3(0.25, 0.28, 0.35);     // muted dark blue-grey
                }

                panel_bg = mix(panel_bg, dot_col, a);
            }
        }

        // Blend the panel over the transparent background
        col = mix(col, panel_bg, panel_a);
        alpha = max(alpha, panel_a * 0.90);
    }

    // ==================== RIGHT COMPONENT (placeholder) ====================
    // Small pill on the right side to demonstrate the multi-section pattern.
    float right_cx = resolution.x - 24.0;
    float right_w = 16.0;

    float right_sdf = sdCapsule(
        p, vec2(right_cx - right_w, hh), vec2(right_cx + right_w, hh), 8.0
    );
    float right_a = 1.0 - smoothstep(0.0, 1.0, right_sdf);

    if (right_a > 0.0) {
        vec3 right_bg = vec3(0.085, 0.095, 0.110);

        // Small dot inside as placeholder content
        float dot_d = sdCircle(p - vec2(right_cx, hh), 3.0);
        float dot_a = 1.0 - smoothstep(0.0, 1.0, dot_d);
        right_bg = mix(right_bg, vec3(0.30, 0.32, 0.40), dot_a);

        col = mix(col, right_bg, right_a);
        alpha = max(alpha, right_a * 0.88);
    }

    // ==================== OUTPUT ====================
    color = vec4(col, alpha);
}
