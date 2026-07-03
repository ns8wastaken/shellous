use crate::renderer::programs::rect::{
    Color, CornerShape, Corners, FillMode, LogicalInset, Mat3, RectProgram, RoundedRectStyle,
};

// ==================== DRAWING CONSTANTS ====================

const PANEL_W: f32 = 260.0;           // panel width from left edge
const START_X: f32 = 20.0;            // first workspace indicator x offset
const SPACING: f32 = 22.0;            // spacing between indicators
const DOT_R: f32 = 2.5;               // inactive dot radius
const CAP_R: f32 = 3.5;               // capsule end radius (active)
const CAP_HALF: f32 = 5.5;            // capsule half-length (active)
const STROKE_H: f32 = 1.5;            // bottom border stroke height

// ==================== PANEL BACKGROUND ====================

fn draw_background(rect: &RectProgram, surface_w: f32, surface_h: f32, panel_h: f32) {
    let style = RoundedRectStyle {
        fill: Color { r: 0.085, g: 0.095, b: 0.110, a: 1.0 },
        fill_mode: FillMode::Solid,
        corners: Corners {
            tl: CornerShape::Convex,
            tr: CornerShape::Concave,
            br: CornerShape::Convex,
            bl: CornerShape::Concave,
        },
        radius: Corners { tl: 0.0, tr: 12.0, br: 12.0, bl: 10.0 },
        logical_inset: LogicalInset { right: 10.0, bottom: 10.0, ..Default::default() },
        softness: 0.85,
        ..Default::default()
    };

    rect.draw(surface_w, surface_h, PANEL_W, panel_h, &style, Mat3::identity());
}

// ==================== WORKSPACE INDICATORS ====================

fn draw_active_indicator(
    rect: &RectProgram,
    surface_w: f32,
    surface_h: f32,
    cx: f32,
    elem_y: f32,
    hover: bool,
) {
    let w = (CAP_HALF + CAP_R) * 2.0;
    let h = CAP_R * 2.0;
    let rx = cx - w * 0.5;
    let ry = elem_y - h * 0.5;

    // Main capsule
    let style = RoundedRectStyle {
        fill: Color { r: 0.48, g: 0.62, b: 0.82, a: 1.0 },
        fill_mode: FillMode::Solid,
        radius: Corners { tl: CAP_R, tr: CAP_R, br: CAP_R, bl: CAP_R },
        softness: 0.85,
        ..Default::default()
    };
    rect.draw(surface_w, surface_h, w, h, &style, Mat3::translation(rx, ry));

    // Inner highlight
    let iw = (CAP_HALF * 0.6 + CAP_R * 0.6) * 2.0;
    let ih = (CAP_R * 0.6) * 2.0;
    let inner_style = RoundedRectStyle {
        fill: Color { r: 0.10, g: 0.12, b: 0.14, a: 0.5 },
        fill_mode: FillMode::Solid,
        radius: Corners { tl: CAP_R * 0.6, tr: CAP_R * 0.6, br: CAP_R * 0.6, bl: CAP_R * 0.6 },
        softness: 0.85,
        ..Default::default()
    };
    rect.draw(
        surface_w, surface_h, iw, ih, &inner_style,
        Mat3::translation(cx - iw * 0.5, elem_y - ih * 0.5),
    );

    // Hover glow
    if hover {
        let gw = (CAP_HALF + CAP_R + 3.0) * 2.0;
        let gh = (CAP_R + 3.0) * 2.0;
        let glow_style = RoundedRectStyle {
            fill: Color { r: 0.55, g: 0.70, b: 0.90, a: 0.12 },
            fill_mode: FillMode::Solid,
            radius: Corners { tl: CAP_R + 3.0, tr: CAP_R + 3.0, br: CAP_R + 3.0, bl: CAP_R + 3.0 },
            softness: 1.5,
            ..Default::default()
        };
        rect.draw(
            surface_w, surface_h, gw, gh, &glow_style,
            Mat3::translation(cx - gw * 0.5, elem_y - gh * 0.5),
        );
    }
}

fn draw_inactive_indicator(
    rect: &RectProgram,
    surface_w: f32,
    surface_h: f32,
    cx: f32,
    elem_y: f32,
    hover: bool,
) {
    let d = DOT_R * 2.0;
    let rx = cx - DOT_R;
    let ry = elem_y - DOT_R;

    let dot_color = if hover {
        Color { r: 0.35, g: 0.40, b: 0.50, a: 1.0 }
    } else {
        Color { r: 0.25, g: 0.28, b: 0.35, a: 1.0 }
    };

    let style = RoundedRectStyle {
        fill: dot_color,
        fill_mode: FillMode::Solid,
        radius: Corners { tl: DOT_R, tr: DOT_R, br: DOT_R, bl: DOT_R },
        softness: 0.85,
        ..Default::default()
    };
    rect.draw(surface_w, surface_h, d, d, &style, Mat3::translation(rx, ry));

    // Hover glow
    if hover {
        let gd = (DOT_R + 3.0) * 2.0;
        let glow_style = RoundedRectStyle {
            fill: Color { r: 0.40, g: 0.50, b: 0.65, a: 0.10 },
            fill_mode: FillMode::Solid,
            radius: Corners { tl: DOT_R + 3.0, tr: DOT_R + 3.0, br: DOT_R + 3.0, bl: DOT_R + 3.0 },
            softness: 1.5,
            ..Default::default()
        };
        rect.draw(
            surface_w, surface_h, gd, gd, &glow_style,
            Mat3::translation(cx - (DOT_R + 3.0), elem_y - (DOT_R + 3.0)),
        );
    }
}

fn draw_workspace_indicators(
    rect: &RectProgram,
    surface_w: f32,
    surface_h: f32,
    ws_count: usize,
    active_slot: i32,
    hover_slot: i32,
) {
    let elem_y = surface_h * 0.5;

    for i in 0..ws_count.min(20) {
        let cx = START_X + i as f32 * SPACING;

        // Stop if this element would extend past the panel edge
        if cx + CAP_HALF + CAP_R > PANEL_W {
            break;
        }

        if i as i32 == active_slot {
            draw_active_indicator(rect, surface_w, surface_h, cx, elem_y, i as i32 == hover_slot);
        } else {
            draw_inactive_indicator(rect, surface_w, surface_h, cx, elem_y, i as i32 == hover_slot);
        }
    }
}

// ==================== BORDER STROKE ====================

fn draw_border_stroke(rect: &RectProgram, surface_w: f32, surface_h: f32) {
    let style = RoundedRectStyle {
        fill: Color { r: 0.50, g: 0.60, b: 0.78, a: 0.55 },
        fill_mode: FillMode::Solid,
        radius: Corners { tl: 0.0, tr: 0.0, br: 0.0, bl: 0.0 },
        softness: 0.5,
        ..Default::default()
    };
    rect.draw(surface_w, surface_h, PANEL_W, STROKE_H, &style, Mat3::translation(0.0, 0.0));
}

// ==================== RIGHT PILL (placeholder) ====================

fn draw_right_pill(rect: &RectProgram, surface_w: f32, surface_h: f32) {
    let right_cx = surface_w - 24.0;
    let right_w = 16.0;
    let elem_y = surface_h * 0.5;

    // Background capsule
    let style = RoundedRectStyle {
        fill: Color { r: 0.085, g: 0.095, b: 0.110, a: 1.0 },
        fill_mode: FillMode::Solid,
        radius: Corners { tl: 8.0, tr: 8.0, br: 8.0, bl: 8.0 },
        softness: 0.85,
        ..Default::default()
    };
    rect.draw(
        surface_w, surface_h, right_w * 2.0, 16.0, &style,
        Mat3::translation(right_cx - right_w, elem_y - 8.0),
    );

    // Small dot inside
    let dot_style = RoundedRectStyle {
        fill: Color { r: 0.30, g: 0.32, b: 0.40, a: 1.0 },
        fill_mode: FillMode::Solid,
        radius: Corners { tl: 3.0, tr: 3.0, br: 3.0, bl: 3.0 },
        softness: 0.85,
        ..Default::default()
    };
    rect.draw(
        surface_w, surface_h, 6.0, 6.0, &dot_style,
        Mat3::translation(right_cx - 3.0, elem_y - 3.0),
    );
}

// ==================== PUBLIC ENTRY POINT ====================

/// Draw the entire bar — panel background, workspace indicators, border, and right pill.
pub fn draw_bar_panel(
    rect: &RectProgram,
    surface_w: f32,
    surface_h: f32,
    ws_count: usize,
    active_slot: i32,
    hover_slot: i32,
) {
    let panel_h = surface_h;

    draw_background(rect, surface_w, surface_h, panel_h);
    draw_workspace_indicators(rect, surface_w, surface_h, ws_count, active_slot, hover_slot);
    draw_border_stroke(rect, surface_w, surface_h);
    draw_right_pill(rect, surface_w, surface_h);
}
