use crate::canvas::DrawingSurface;
use crate::components::row::Row;
use crate::renderer::programs::rect::{
    Color, CornerShape, Corners, FillMode, LogicalInset, Mat3, RectStyle,
};
use crate::services::workspace::{WorkspaceHandle, WorkspaceSnapshot};
use crate::ui::{Element, RenderContext};

const WORKSPACE_SPACING: f32 = 8.0;
const WORKSPACE_R: f32 = 5.5;
const WORKSPACE_INACTIVE_W: f32 = WORKSPACE_R * 2.0;
const WORKSPACE_ACTIVE_W: f32 = WORKSPACE_INACTIVE_W * 3.0;
const PANEL_OFFSET_Y: f32 = 18.0;

/// Three panel-extent values are derived from one base — `rounding`. They're
/// kept as a struct so the relationship reads clearly at the call site:
///
/// ```text
/// rounding = panel_h / 2
/// start_x  = rounding      (= panel_h / 2)
/// end_pad  = rounding * 2  (= panel_h)
/// ```
///
/// So with the typical `panel_h = 36`: `start_x = 18`, `end_pad = 36`.
/// Tweak `rounding` (the single knob) and the whole panel recomposes.
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

/// Vertical y for the indicator row: cell-local origin sits here such that
/// the dot's geometric center lands on the panel's vertical midline.
fn indicator_row_y(surface_h: f32) -> f32 {
    (surface_h - PANEL_OFFSET_Y) * 0.5 - WORKSPACE_R
}

/// Index of the active workspace inside the snapshot, if any.
fn active_index(snap: &WorkspaceSnapshot) -> Option<usize> {
    snap.workspaces.iter().position(|w| w.id == snap.active_id)
}

pub struct LeftPanel {
    handle: WorkspaceHandle,
}

impl LeftPanel {
    pub fn new(handle: WorkspaceHandle) -> Self {
        Self { handle }
    }
}

impl Element for LeftPanel {
    fn draw(&self, surface: &dyn DrawingSurface, ctx: &RenderContext) {
        let layout = PanelLayout::from_surface(ctx.surface_h);
        let snap = self.handle.snapshot();
        let active_idx = active_index(&snap);

        let mut row = Row::new()
            .at(layout.start_x, indicator_row_y(ctx.surface_h))
            .spacing(WORKSPACE_SPACING);
        for (i, ws) in snap.workspaces.iter().enumerate() {
            row = row.add(Box::new(WorkspaceIndicatorCell {
                workspace_id: ws.id,
                is_active: active_idx == Some(i),
            }));
        }

        // Panel width: leading offset (start_x) + the row's own computed
        // width (sum of cell widths + (n-1) spacings) + trailing padding (end_pad).
        let panel_w = layout.start_x + row.size().0 + layout.end_pad;
        draw_background(surface, ctx.surface_w, ctx.surface_h, layout.panel_h, panel_w);
        row.draw(surface, ctx);
    }

    fn on_click(&self, x: f32, y: f32, ctx: &RenderContext) -> bool {
        let layout = PanelLayout::from_surface(ctx.surface_h);
        let snap = self.handle.snapshot();
        let active_idx = active_index(&snap);

        let mut row = Row::new()
            .at(layout.start_x, indicator_row_y(ctx.surface_h))
            .spacing(WORKSPACE_SPACING);
        for (i, ws) in snap.workspaces.iter().enumerate() {
            row = row.add(Box::new(WorkspaceIndicatorCell {
                workspace_id: ws.id,
                is_active: active_idx == Some(i),
            }));
        }

        // Mirror the panel-w computation from `draw` so future click
        // affordances can rely on the same bbox. Currently unused by
        // `row.on_click`; keep in step in case a background-affordance
        // click handler is added.
        let _panel_w = layout.start_x + row.size().0 + layout.end_pad;
        row.on_click(x, y, ctx)
    }
}

// ==================== WORKSPACE INDICATOR CELL ====================

/// One row cell. Cell-local origin (0, 0) is where Row's TranslatedCanvas
/// maps to on screen — the cell draws at identity.
struct WorkspaceIndicatorCell {
    workspace_id: i32,
    is_active: bool,
}

impl WorkspaceIndicatorCell {
    fn width(&self) -> f32 {
        if self.is_active {
            WORKSPACE_ACTIVE_W
        } else {
            WORKSPACE_INACTIVE_W
        }
    }
}

impl Element for WorkspaceIndicatorCell {
    fn size(&self) -> (f32, f32) {
        (self.width(), WORKSPACE_R * 2.0)
    }

    fn draw(&self, surface: &dyn DrawingSurface, ctx: &RenderContext) {
        let fill = if self.is_active {
            Color { r: 0.10, g: 0.12, b: 0.14, a: 1.0 }
        } else {
            Color { r: 0.25, g: 0.28, b: 0.35, a: 1.0 }
        };
        let style = RectStyle {
            fill,
            fill_mode: FillMode::Solid,
            radius: Corners {
                tl: WORKSPACE_R, tr: WORKSPACE_R, br: WORKSPACE_R, bl: WORKSPACE_R,
            },
            softness: 0.85,
            ..Default::default()
        };
        surface.draw_rect(
            ctx.surface_w,
            ctx.surface_h,
            self.width(),
            WORKSPACE_R * 2.0,
            &style,
            Mat3::identity(),
        );
    }

    fn on_click(&self, _x: f32, _y: f32, ctx: &RenderContext) -> bool {
        ctx.state.compositor.activate_workspace(self.workspace_id);
        true
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
