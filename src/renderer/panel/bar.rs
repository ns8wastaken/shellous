use crate::action::Action;
use crate::bar;
use crate::renderer::panel::{Panel, RenderContext};
use crate::renderer::programs::rect::{
    Color, CornerShape, Corners, FillMode, LogicalInset, Mat3, RectProgram, RectStyle,
};

// ==================== DRAWING CONSTANTS ====================

const START_X: f32 = 20.0;
const SPACING: f32 = 22.0;
const DOT_R: f32 = 2.5;
const CAP_R: f32 = 3.5;
const CAP_HALF: f32 = 5.5;

// ==================== BAR PANEL ====================

pub struct BarPanel {
    pub width: f32,
}

impl Default for BarPanel {
    fn default() -> Self {
        Self { width: 260.0 }
    }
}

impl Panel for BarPanel {
    fn draw(&self, rect: &RectProgram, ctx: &RenderContext) {
        let (ws_count, active_slot) = {
            let bar = ctx.state.bar.lock().unwrap();
            let active_slot = bar
                .workspaces
                .iter()
                .position(|w| w.id == bar.active_id)
                .map(|i| i as i32)
                .unwrap_or(-1);
            (bar.workspaces.len(), active_slot)
        };

        let hover_slot = ctx
            .pointer_pos
            .and_then(|(px, py)| {
                let buttons = bar::button_layout(ws_count, ctx.surface_h);
                bar::hit_test(&buttons, px as f32, py as f32)
            })
            .map(|i| i as i32)
            .unwrap_or(-1);

        let panel_h = ctx.surface_h - 18.0;

        draw_background(rect, ctx.surface_w, ctx.surface_h, panel_h, self.width);
        draw_workspace_indicators(
            rect,
            ctx.surface_w,
            ctx.surface_h,
            self.width,
            panel_h,
            ws_count,
            active_slot,
            hover_slot,
        );
        draw_right_pill(rect, ctx.surface_w, ctx.surface_h);
    }

    fn on_click(&self, x: f32, y: f32, ctx: &RenderContext) -> Action {
        let bar = ctx.state.bar.lock().unwrap();
        let ws_count = bar.workspaces.len();
        let buttons = bar::button_layout(ws_count, ctx.surface_h);

        match bar::hit_test(&buttons, x, y) {
            Some(idx) => {
                let id = bar.workspaces[idx].id;
                Action::SwitchWorkspace(id)
            }
            None => Action::None,
        }
    }
}

// ==================== PANEL BACKGROUND ====================

fn draw_background(
    rect: &RectProgram,
    surface_w: f32,
    surface_h: f32,
    panel_h: f32,
    panel_w: f32,
) {
    let style = RectStyle {
        fill: Color { r: 0.085, g: 0.095, b: 0.110, a: 1.0 },
        fill_mode: FillMode::Solid,
        corners: Corners {
            tl: CornerShape::Convex,
            tr: CornerShape::Concave,
            br: CornerShape::Convex,
            bl: CornerShape::Concave,
        },
        radius: Corners { tl: 0.0, tr: 12.0, br: 12.0, bl: 18.0 },
        logical_inset: LogicalInset { right: 10.0, bottom: 18.0, ..Default::default() },
        ..Default::default()
    };
    rect.draw(surface_w, surface_h, panel_w, panel_h + 18.0, &style, Mat3::identity());
}

// ==================== WORKSPACE INDICATORS ====================

fn draw_active_indicator(
    rect: &RectProgram,
    surface_w: f32,
    surface_h: f32,
    elem_x: f32,
    elem_y: f32,
    hover: bool,
) {
    let iw = (CAP_HALF * 0.6 + CAP_R * 0.6) * 2.0;
    let ih = (CAP_R * 0.6) * 2.0;
    let inner_style = RectStyle {
        fill: Color { r: 0.10, g: 0.12, b: 0.14, a: 0.5 },
        fill_mode: FillMode::Solid,
        radius: Corners { tl: CAP_R * 0.6, tr: CAP_R * 0.6, br: CAP_R * 0.6, bl: CAP_R * 0.6 },
        softness: 0.85,
        ..Default::default()
    };
    rect.draw(
        surface_w,
        surface_h,
        iw,
        ih,
        &inner_style,
        Mat3::translation(elem_x - iw * 0.5, elem_y - ih * 0.5),
    );

    if hover {
        let gw = (CAP_HALF + CAP_R + 3.0) * 2.0;
        let gh = (CAP_R + 3.0) * 2.0;
        let glow_style = RectStyle {
            fill: Color { r: 0.55, g: 0.70, b: 0.90, a: 0.12 },
            fill_mode: FillMode::Solid,
            radius: Corners { tl: CAP_R + 3.0, tr: CAP_R + 3.0, br: CAP_R + 3.0, bl: CAP_R + 3.0 },
            softness: 1.5,
            ..Default::default()
        };
        rect.draw(
            surface_w,
            surface_h,
            gw,
            gh,
            &glow_style,
            Mat3::translation(elem_x - gw * 0.5, elem_y - gh * 0.5),
        );
    }
}

fn draw_inactive_indicator(
    rect: &RectProgram,
    surface_w: f32,
    surface_h: f32,
    elem_x: f32,
    elem_y: f32,
    hover: bool,
) {
    let d = DOT_R * 2.0;
    let rx = elem_x - DOT_R;
    let ry = elem_y - DOT_R;

    let dot_color = if hover {
        Color { r: 0.35, g: 0.40, b: 0.50, a: 1.0 }
    } else {
        Color { r: 0.25, g: 0.28, b: 0.35, a: 1.0 }
    };

    let style = RectStyle {
        fill: dot_color,
        fill_mode: FillMode::Solid,
        radius: Corners { tl: DOT_R, tr: DOT_R, br: DOT_R, bl: DOT_R },
        softness: 0.85,
        ..Default::default()
    };
    rect.draw(surface_w, surface_h, d, d, &style, Mat3::translation(rx, ry));

    if hover {
        let gd = (DOT_R + 3.0) * 2.0;
        let glow_style = RectStyle {
            fill: Color { r: 0.40, g: 0.50, b: 0.65, a: 0.10 },
            fill_mode: FillMode::Solid,
            radius: Corners { tl: DOT_R + 3.0, tr: DOT_R + 3.0, br: DOT_R + 3.0, bl: DOT_R + 3.0 },
            softness: 1.5,
            ..Default::default()
        };
        rect.draw(
            surface_w, surface_h, gd, gd, &glow_style,
            Mat3::translation(elem_x - (DOT_R + 3.0), elem_y - (DOT_R + 3.0)),
        );
    }
}

fn draw_workspace_indicators(
    rect: &RectProgram,
    surface_w: f32,
    surface_h: f32,
    panel_w: f32,
    panel_h: f32,
    ws_count: usize,
    active_slot: i32,
    hover_slot: i32,
) {
    let elem_y = panel_h * 0.5;

    for i in 0..ws_count.min(20) {
        let elem_x = START_X + i as f32 * SPACING;

        if elem_x + CAP_HALF + CAP_R > panel_w {
            break;
        }

        if i as i32 == active_slot {
            draw_active_indicator(
                rect, surface_w,
                surface_h,
                elem_x,
                elem_y,
                i as i32 == hover_slot
            );
        } else {
            draw_inactive_indicator(
                rect,
                surface_w,
                surface_h,
                elem_x,
                elem_y,
                i as i32 == hover_slot
            );
        }
    }
}

// ==================== RIGHT PILL (placeholder) ====================

fn draw_right_pill(rect: &RectProgram, surface_w: f32, surface_h: f32) {
    let right_cx = surface_w - 24.0;
    let right_w = 16.0;
    let elem_y = surface_h * 0.5;

    let style = RectStyle {
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

    let dot_style = RectStyle {
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
