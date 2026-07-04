use crate::ui::Action;
use crate::components::bar::state;
use crate::renderer::programs::rect::{
    Color, CornerShape, Corners, FillMode, LogicalInset, Mat3, RectProgram, RectStyle,
};
use crate::ui::{Element, RenderContext};

const WORKSPACE_SPACING: f32 = 8.0;
const WORKSPACE_R: f32 = 6.0;
const WORKSPACE_INACTIVE_W: f32 = WORKSPACE_R * 2.0;
const WORKSPACE_ACTIVE_W: f32 = WORKSPACE_INACTIVE_W * 3.0;

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

        let hover_slot = ctx
            .pointer_pos
            .and_then(|(px, py)| {
                let buttons = state::button_layout(ws_count, ctx.surface_h);
                state::hit_test(&buttons, px as f32, py as f32)
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
    }

    fn on_click(&self, x: f32, y: f32, ctx: &RenderContext) -> Action {
        let bar = ctx.state.bar.lock().unwrap();
        let ws_count = bar.workspaces.len();
        let buttons = state::button_layout(ws_count, ctx.surface_h);

        match state::hit_test(&buttons, x, y) {
            Some(idx) => {
                let id = bar.workspaces[idx].id;
                Action::SwitchWorkspace(id)
            }
            None => Action::None,
        }
    }
}

fn draw_active_indicator(
    rect: &RectProgram,
    surface_w: f32,
    surface_h: f32,
    elem_x: f32,
    elem_y: f32,
    hover: bool,
) {
    let inner_style = RectStyle {
        fill: Color { r: 0.10, g: 0.12, b: 0.14, a: 0.5 },
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
        &inner_style,
        Mat3::translation(elem_x - WORKSPACE_ACTIVE_W * 0.5, elem_y - WORKSPACE_ACTIVE_W * 0.5),
    );

    if hover {
        let gw = WORKSPACE_ACTIVE_W + 2.0;
        let gh = WORKSPACE_R * 2.0 + 2.0;
        let gr = gh * 0.5;
        let glow_style = RectStyle {
            fill: Color { r: 0.55, g: 0.70, b: 0.90, a: 0.12 },
            fill_mode: FillMode::Solid,
            radius: Corners { tl: gr, tr: gr, br: gr, bl: gr },
            softness: 1.5,
            ..Default::default()
        };
        rect.draw(
            surface_w,
            surface_h,
            gw,
            gh,
            &glow_style,
            Mat3::translation(elem_x - gr, elem_y - gr),
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
    let dot_color = if hover {
        Color { r: 0.35, g: 0.40, b: 0.50, a: 1.0 }
    } else {
        Color { r: 0.25, g: 0.28, b: 0.35, a: 1.0 }
    };

    let style = RectStyle {
        fill: dot_color,
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
        Mat3::translation(elem_x - WORKSPACE_R, elem_y - WORKSPACE_R)
    );

    if hover {
        let gsize = WORKSPACE_INACTIVE_W + 2.0;
        let gr = gsize * 0.5;
        let glow_style = RectStyle {
            fill: Color { r: 0.40, g: 0.50, b: 0.65, a: 0.10 },
            fill_mode: FillMode::Solid,
            radius: Corners { tl: gr, tr: gr, br: gr, bl: gr },
            softness: 1.5,
            ..Default::default()
        };
        rect.draw(
            surface_w, surface_h, gsize, gsize, &glow_style,
            Mat3::translation(elem_x - gr, elem_y - gr),
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

    for i in 0..ws_count {
        let elem_x = 18.0 + (WORKSPACE_SPACING + WORKSPACE_INACTIVE_W) * i as f32;

        // if elem_x + CAP_HALF + CAP_R > panel_w {
        //     break;
        // }

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
