use crate::components::canvas::DrawingSurface;
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
    width: Animated<f32>,
}

impl WorkspaceDot {
    fn new(workspace_id: i32, is_active: bool) -> Self {
        let initial = if is_active { WORKSPACE_ACTIVE_W } else { WORKSPACE_INACTIVE_W };
        Self {
            workspace_id,
            width: Animated::new(initial)
                .with_duration(0.26)
                .with_easing(Easing::EaseOutCubic),
        }
    }
}

// ==================== LEFT PANEL ====================

pub struct LeftPanel {
    handle: WorkspaceHandle,
    prev_active_id: i32,
    prev_workspace_ids: Vec<i32>,
    dots: Vec<WorkspaceDot>,
}

impl LeftPanel {
    pub fn new(handle: WorkspaceHandle) -> Self {
        let snap = handle.snapshot();
        let dots: Vec<_> = snap
            .workspaces
            .iter()
            .map(|ws| WorkspaceDot::new(ws.id, ws.id == snap.active_id))
            .collect();
        Self {
            handle,
            prev_active_id: snap.active_id,
            prev_workspace_ids: snap.workspaces.iter().map(|w| w.id).collect(),
            dots,
        }
    }
}

impl Element for LeftPanel {
    fn tick_animations(&mut self, absolute_time: f32) -> bool {
        let snap = self.handle.snapshot();

        // Detect active workspace change
        if snap.active_id != self.prev_active_id {
            // Old active → inactive width
            for dot in &mut self.dots {
                if dot.workspace_id == self.prev_active_id {
                    dot.width
                        .set_target(WORKSPACE_INACTIVE_W, absolute_time);
                }
            }
            // New active → active width
            for dot in &mut self.dots {
                if dot.workspace_id == snap.active_id {
                    dot.width
                        .set_target(WORKSPACE_ACTIVE_W, absolute_time);
                }
            }
            self.prev_active_id = snap.active_id;
        }

        // Detect workspace list changes (add/remove)
        let cur_ids: Vec<i32> = snap.workspaces.iter().map(|w| w.id).collect();
        if cur_ids != self.prev_workspace_ids {
            // Added workspaces
            for ws in &snap.workspaces {
                if !self.prev_workspace_ids.contains(&ws.id) {
                    self.dots.push(WorkspaceDot::new(ws.id, ws.id == snap.active_id));
                }
            }
            // Removed workspaces
            self.dots
                .retain(|dot| cur_ids.contains(&dot.workspace_id));
            self.prev_workspace_ids = cur_ids;
        }

        // Check if any dot is still animating
        let mut any_active = false;
        for dot in &self.dots {
            if !dot.width.is_idle(absolute_time) {
                any_active = true;
                break;
            }
        }
        any_active
    }

    fn draw(&self, surface: &dyn DrawingSurface, ctx: &RenderContext) {
        let layout = PanelLayout::from_surface(ctx.surface_h);
        let y = indicator_row_y(ctx.surface_h);

        // First pass — compute total width for background
        let mut x = layout.start_x;
        for dot in &self.dots {
            x += dot.width.value(ctx.absolute_time) + WORKSPACE_SPACING;
        }
        let panel_w = x + layout.end_pad;

        // Background (behind dots)
        draw_background(surface, ctx.surface_w, ctx.surface_h, layout.panel_h, panel_w);

        // Second pass — draw dots on top
        x = layout.start_x;
        for dot in &self.dots {
            let w = dot.width.value(ctx.absolute_time);
            let fill = if dot.workspace_id == self.prev_active_id {
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
                Mat3::translation(x, y),
            );
            x += w + WORKSPACE_SPACING;
        }
    }

    fn on_click(&self, click_x: f32, click_y: f32, ctx: &RenderContext) -> bool {
        let layout = PanelLayout::from_surface(ctx.surface_h);
        let mut x = layout.start_x;
        let y = indicator_row_y(ctx.surface_h);

        for dot in &self.dots {
            let w = dot.width.value(ctx.absolute_time);
            let h = WORKSPACE_R * 2.0;
            if click_x >= x && click_x <= x + w && click_y >= y && click_y <= y + h {
                ctx.state.compositor.activate_workspace(dot.workspace_id);
                return true;
            }
            x += w + WORKSPACE_SPACING;
        }
        false
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
