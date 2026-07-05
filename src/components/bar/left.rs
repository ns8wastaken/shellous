use crate::components::canvas::DrawingSurface;
use crate::components::layout::padding::Padding;
use crate::components::layout::row::Row;
use crate::components::ui::{Element, RenderContext};
use crate::renderer::animation::Animated;
use crate::renderer::animation::easing::Easing;
use crate::renderer::programs::rect::{
    Color, CornerShape, Corners, FillMode, LogicalInset, Mat3, RectStyle,
};
use crate::services::workspace::WorkspaceSnapshot;

const WORKSPACE_SPACING: f32 = 8.0;
const WORKSPACE_R: f32 = 5.5;
const WORKSPACE_INACTIVE_W: f32 = WORKSPACE_R * 2.0;
const WORKSPACE_ACTIVE_W: f32 = WORKSPACE_INACTIVE_W * 3.0;

// ==================== WORKSPACE DOT ====================

struct WorkspaceDot {
    workspace_id: i32,
    is_active: bool,
    width: Animated<f32>,
}

impl WorkspaceDot {
    fn new(workspace_id: i32) -> Self {
        Self {
            workspace_id,
            is_active: false,
            width: Animated::new(WORKSPACE_INACTIVE_W)
                .with_duration(0.26)
                .with_easing(Easing::EaseOutCubic),
        }
    }
}

impl Element for WorkspaceDot {
    fn update(&mut self, snapshot: &WorkspaceSnapshot) {
        self.is_active = snapshot.active_id == self.workspace_id;
    }

    fn tick_animations(&mut self, absolute_time: f32) -> bool {
        let target = if self.is_active { WORKSPACE_ACTIVE_W } else { WORKSPACE_INACTIVE_W };
        if target != self.width.target() {
            self.width.set_target(target, absolute_time);
        }

        self.width.tick(absolute_time)
    }

    fn draw(&self, surface: &dyn DrawingSurface, ctx: &RenderContext) {
        let w = self.width.value();
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
        let w = self.width.value();
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
        (self.width.value(), WORKSPACE_R * 2.0)
    }
}

// ==================== LEFT PANEL ====================

pub struct LeftPanel {
    padded_row: Padding,
    panel_width: Animated<f32>,
    total_horizontal_padding: f32,
    bottom_offset: f32,
    prev_workspace_ids: Vec<i32>,
}

impl LeftPanel {
    pub fn new(bottom_offset: f32) -> Self {
        let top = 15.0 - WORKSPACE_R;
        let corner_r = 30.0 / 2.0;

        let row = Row::new().spacing(WORKSPACE_SPACING);
        let padded_row = Padding::new(Box::new(row))
            .left(corner_r)
            .top(top)
            .right(corner_r * 2.0);

        Self {
            padded_row,
            panel_width: Animated::new(corner_r * 3.0)
                .with_duration(0.26)
                .with_easing(Easing::EaseOutCubic),
            total_horizontal_padding: corner_r * 3.0,
            bottom_offset,
            prev_workspace_ids: Vec::new(),
        }
    }
}

impl Element for LeftPanel {
    fn update(&mut self, snapshot: &WorkspaceSnapshot) {
        let mut cur_ids: Vec<i32> = snapshot.workspaces.iter().map(|w| w.id).collect();
        cur_ids.sort_unstable();
        if cur_ids != self.prev_workspace_ids {
            let row = self.padded_row.child
                .as_any_mut()
                .and_then(|a| a.downcast_mut::<Row>())
                .expect("LeftPanel child should be a Row");

            let old = std::mem::take(&mut row.children);
            let mut by_id: Vec<(i32, Box<dyn Element>)> = old
                .into_iter()
                .filter_map(|c| c.id().map(|id| (id, c)))
                .collect();

            let mut sorted = snapshot.workspaces.clone();
            sorted.sort_by_key(|ws| ws.id);

            for ws in &sorted {
                match by_id.iter().position(|(id, _)| *id == ws.id) {
                    Some(idx) => row.children.push(by_id.remove(idx).1),
                    None => row.children.push(Box::new(WorkspaceDot::new(ws.id))),
                }
            }
            self.prev_workspace_ids = cur_ids;
        }

        self.padded_row.update(snapshot);
    }

    fn tick_animations(&mut self, absolute_time: f32) -> bool {
        let row_animating = self.padded_row.tick_animations(absolute_time);

        let target = self.padded_row.child.size().0 + self.total_horizontal_padding;
        if target != self.panel_width.target() {
            self.panel_width.set_target(target, absolute_time);
        }
        let panel_animating = self.panel_width.tick(absolute_time);

        row_animating || panel_animating
    }

    fn draw(&self, surface: &dyn DrawingSurface, ctx: &RenderContext) {
        draw_background(
            surface,
            ctx.surface_w,
            ctx.surface_h,
            self.panel_width.value(),
            self.bottom_offset
        );

        self.padded_row.draw(surface, ctx);
    }

    fn on_click(&self, x: f32, y: f32, ctx: &RenderContext) -> bool {
        self.padded_row.on_click(x, y, ctx)
    }
}

// ==================== PANEL BACKGROUND ====================

fn draw_background(
    surface: &dyn DrawingSurface,
    surface_w: f32,
    surface_h: f32,
    panel_w: f32,
    bottom_offset: f32,
) {
    let corner_r = (surface_h - 18.0) / 2.0;
    let style = RectStyle {
        fill: Color { r: 0.085, g: 0.095, b: 0.110, a: 1.0 },
        fill_mode: FillMode::Solid,
        corners: Corners {
            tl: CornerShape::Convex,
            tr: CornerShape::Concave,
            br: CornerShape::Convex,
            bl: CornerShape::Concave,
        },
        radius: Corners { tl: 0.0, tr: corner_r, br: corner_r, bl: bottom_offset },
        logical_inset: LogicalInset { right: corner_r, bottom: bottom_offset, ..Default::default() },
        ..Default::default()
    };
    surface.draw_rect(
        surface_w,
        surface_h,
        panel_w,
        surface_h,
        &style,
        Mat3::identity(),
    );
}
