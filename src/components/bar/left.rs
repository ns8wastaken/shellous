use std::cell::Cell;

use crate::components::canvas::{DrawingSurface, TranslatedCanvas};
use crate::components::row::Row;
use crate::components::ui::{Element, RenderContext};
use crate::renderer::animation::Animated;
use crate::renderer::animation::easing::Easing;
use crate::renderer::programs::rect::{
    Color, CornerShape, Corners, FillMode, LogicalInset, Mat3, RectStyle,
};
use crate::services::workspace::WorkspaceHandle;

const WORKSPACE_SPACING: f32 = 8.0;
const WORKSPACE_R: f32 = 5.5;
const WORKSPACE_INACTIVE_W: f32 = WORKSPACE_R * 2.0;
const WORKSPACE_ACTIVE_W: f32 = WORKSPACE_INACTIVE_W * 3.0;
const PANEL_OFFSET_Y: f32 = 18.0;

// ==================== PANEL LAYOUT ====================

struct PanelLayout {
    panel_h: f32,
    start_x: f32,
    end_pad: f32,
}

impl PanelLayout {
    fn from_surface(surface_h: f32) -> Self {
        let panel_h = surface_h - PANEL_OFFSET_Y;
        let rounding = panel_h * 0.5;
        Self {
            panel_h,
            start_x: rounding,
            end_pad: rounding * 2.0,
        }
    }
}

fn indicator_row_y(surface_h: f32) -> f32 {
    (surface_h - PANEL_OFFSET_Y) * 0.5 - WORKSPACE_R
}

// ==================== WORKSPACE DOT ====================

struct WorkspaceDot {
    workspace_id: i32,
    handle: WorkspaceHandle,
    is_active: bool,
    width: Animated<f32>,
    current_width: Cell<f32>,
}

impl WorkspaceDot {
    fn new(workspace_id: i32, is_active: bool, handle: WorkspaceHandle) -> Self {
        let initial = if is_active { WORKSPACE_ACTIVE_W } else { WORKSPACE_INACTIVE_W };
        Self {
            workspace_id,
            handle,
            is_active,
            width: Animated::new(initial)
                .with_duration(0.26)
                .with_easing(Easing::EaseOutCubic),
            current_width: Cell::new(initial),
        }
    }
}

impl Element for WorkspaceDot {
    fn tick_animations(&mut self, absolute_time: f32) -> bool {
        let snap = self.handle.snapshot();
        let is_active_now = snap.active_id == self.workspace_id;
        if is_active_now != self.is_active {
            self.is_active = is_active_now;
            let target = if is_active_now { WORKSPACE_ACTIVE_W } else { WORKSPACE_INACTIVE_W };
            self.width.set_target(target, absolute_time);
        }

        let w = self.width.value(absolute_time);
        self.current_width.set(w);
        !self.width.is_idle(absolute_time)
    }

    fn draw(&self, surface: &dyn DrawingSurface, ctx: &RenderContext) {
        let w = self.width.value(ctx.absolute_time);
        let fill = if self.is_active {
            Color { r: 1.0, g: 0.12, b: 0.14, a: 1.0 }
        } else {
            Color { r: 0.25, g: 0.28, b: 0.35, a: 1.0 }
        };
        let style = RectStyle {
            fill,
            fill_mode: FillMode::Solid,
            radius: Corners {
                tl: WORKSPACE_R,
                tr: WORKSPACE_R,
                br: WORKSPACE_R,
                bl: WORKSPACE_R,
            },
            softness: 0.85,
            ..Default::default()
        };
        surface.draw_rect(
            ctx.surface_w,
            ctx.surface_h,
            w,
            WORKSPACE_R * 2.0,
            &style,
            Mat3::identity(),
        );
    }

    fn on_click(&self, click_x: f32, click_y: f32, ctx: &RenderContext) -> bool {
        let w = self.width.value(ctx.absolute_time);
        let h = WORKSPACE_R * 2.0;
        if click_x >= 0.0 && click_x <= w && click_y >= 0.0 && click_y <= h {
            ctx.state.compositor.activate_workspace(self.workspace_id);
            return true;
        }
        false
    }

    fn id(&self) -> Option<i32> {
        Some(self.workspace_id)
    }

    fn size(&self) -> (f32, f32) {
        (self.current_width.get(), WORKSPACE_R * 2.0)
    }
}

// ==================== LEFT PANEL ====================

pub struct LeftPanel {
    handle: WorkspaceHandle,
    row: Row,
    prev_workspace_ids: Vec<i32>,
}

impl LeftPanel {
    pub fn new(handle: WorkspaceHandle) -> Self {
        let snap = handle.snapshot();
        let ids: Vec<i32> = snap.workspaces.iter().map(|w| w.id).collect();
        let mut row = Row::new().spacing(WORKSPACE_SPACING);
        for ws in &snap.workspaces {
            row.push(Box::new(WorkspaceDot::new(
                ws.id,
                ws.id == snap.active_id,
                handle.clone(),
            )));
        }
        Self {
            handle,
            row,
            prev_workspace_ids: ids,
        }
    }
}

impl Element for LeftPanel {
    fn tick_animations(&mut self, absolute_time: f32) -> bool {
        let snap = self.handle.snapshot();

        // Structural changes — add / remove workspace dots
        let cur_ids: Vec<i32> = snap.workspaces.iter().map(|w| w.id).collect();
        if cur_ids != self.prev_workspace_ids {
            self.row.children_mut().retain(|c| match c.id() {
                Some(id) => cur_ids.contains(&id),
                None => true,
            });
            for ws in &snap.workspaces {
                if !self.prev_workspace_ids.contains(&ws.id) {
                    self.row.push(Box::new(WorkspaceDot::new(
                        ws.id,
                        ws.id == snap.active_id,
                        self.handle.clone(),
                    )));
                }
            }
            self.prev_workspace_ids = cur_ids;
        }

        self.row.tick_animations(absolute_time)
    }

    fn draw(&self, surface: &dyn DrawingSurface, ctx: &RenderContext) {
        let layout = PanelLayout::from_surface(ctx.surface_h);
        let y = indicator_row_y(ctx.surface_h);
        let row_w = self.row.size().0;
        let panel_w = row_w + layout.start_x + layout.end_pad;

        draw_background(surface, ctx.surface_w, ctx.surface_h, layout.panel_h, panel_w);

        let tc = TranslatedCanvas::new(surface, layout.start_x, y);
        self.row.draw(&tc, ctx);
    }

    fn on_click(&self, x: f32, y: f32, ctx: &RenderContext) -> bool {
        let layout = PanelLayout::from_surface(ctx.surface_h);
        let dot_y = indicator_row_y(ctx.surface_h);
        self.row.on_click(x - layout.start_x, y - dot_y, ctx)
    }
}

// ==================== PANEL BACKGROUND ====================

fn draw_background(
    surface: &dyn DrawingSurface,
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
    surface.draw_rect(
        surface_w,
        surface_h,
        panel_w,
        panel_h + 18.0,
        &style,
        Mat3::identity(),
    );
}
