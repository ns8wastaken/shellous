use crate::renderer::programs::rect::{
    Color, CornerShape, Corners, FillMode, LogicalInset, Mat3, RectProgram, RectStyle,
};
use crate::ui::{Element, RenderContext};

const WORKSPACE_SPACING: f32 = 8.0;
const WORKSPACE_R: f32 = 5.5;
const WORKSPACE_INACTIVE_W: f32 = WORKSPACE_R * 2.0;
const WORKSPACE_ACTIVE_W: f32 = WORKSPACE_INACTIVE_W * 3.0;
const PANEL_OFFSET_Y: f32 = 18.0;
const START_X: f32 = 18.0;

/// Returns the x-center of the workspace indicator at the given index.
fn workspace_elem_x(index: usize) -> f32 {
    START_X + (WORKSPACE_SPACING + WORKSPACE_INACTIVE_W) * index as f32
}

pub struct LeftPanel {
    pub width: f32,
}

impl Default for LeftPanel {
    fn default() -> Self {
        Self { width: 260.0 }
    }
}

impl Element for LeftPanel {
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

        let panel_h = ctx.surface_h - PANEL_OFFSET_Y;

        draw_background(rect, ctx.surface_w, ctx.surface_h, panel_h, self.width);
        draw_workspace_indicators(
            rect,
            ctx.surface_w,
            ctx.surface_h,
            panel_h,
            ws_count,
            active_slot,
        );
    }

    fn on_click(&self, x: f32, y: f32, ctx: &RenderContext) -> bool {
        let panel_h = ctx.surface_h - PANEL_OFFSET_Y;
        let cy = panel_h * 0.5;
        let hh = WORKSPACE_R + 2.0;

        let bar = ctx.state.bar.lock().unwrap();
        for i in 0..bar.workspaces.len() {
            let cx = workspace_elem_x(i);
            if x >= cx && x <= cx + WORKSPACE_INACTIVE_W && y >= cy - hh && y <= cy + hh {
                let id = bar.workspaces[i].id;
                drop(bar);
                ctx.state.compositor.switch_workspace(id);
                return true;
            }
        }
        false
    }
}

fn draw_active_indicator(
    rect: &RectProgram,
    surface_w: f32,
    surface_h: f32,
    elem_x: f32,
    elem_y: f32,
) {
    let style = RectStyle {
        fill: Color { r: 0.10, g: 0.12, b: 0.14, a: 1.0 },
        fill_mode: FillMode::Solid,
        radius: Corners { tl: WORKSPACE_R, tr: WORKSPACE_R, br: WORKSPACE_R, bl: WORKSPACE_R },
        softness: 0.85,
        ..Default::default()
    };
    rect.draw(
        surface_w,
        surface_h,
        WORKSPACE_ACTIVE_W,
        WORKSPACE_R * 2.0,
        &style,
        Mat3::translation(elem_x, elem_y - WORKSPACE_R),
    );
}

fn draw_inactive_indicator(
    rect: &RectProgram,
    surface_w: f32,
    surface_h: f32,
    elem_x: f32,
    elem_y: f32,
) {
    let style = RectStyle {
        fill: Color { r: 0.25, g: 0.28, b: 0.35, a: 1.0 },
        fill_mode: FillMode::Solid,
        radius: Corners { tl: WORKSPACE_R, tr: WORKSPACE_R, br: WORKSPACE_R, bl: WORKSPACE_R },
        softness: 0.85,
        ..Default::default()
    };
    rect.draw(
        surface_w,
        surface_h,
        WORKSPACE_INACTIVE_W,
        WORKSPACE_INACTIVE_W,
        &style,
        Mat3::translation(elem_x, elem_y - WORKSPACE_R)
    );
}

fn draw_workspace_indicators(
    rect: &RectProgram,
    surface_w: f32,
    surface_h: f32,
    panel_h: f32,
    ws_count: usize,
    active_slot: i32,
) {
    let elem_y = panel_h * 0.5;

    for i in 0..ws_count {
        let elem_x = workspace_elem_x(i);

        if i as i32 == active_slot {
            draw_active_indicator(rect, surface_w, surface_h, elem_x, elem_y);
        } else {
            draw_inactive_indicator(rect, surface_w, surface_h, elem_x, elem_y);
        }
    }
}

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
        logical_inset: LogicalInset { right: 12.0, bottom: 18.0, ..Default::default() },
        ..Default::default()
    };
    rect.draw(
        surface_w,
        surface_h,
        panel_w,
        panel_h + 18.0,
        &style,
        Mat3::identity()
    );
}
