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
use super::workspace_dot::{WorkspaceDot, WORKSPACE_R};

const WORKSPACE_SPACING: f32 = 8.0;

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
            self.padded_row.sync_children(&cur_ids, &mut |id| {
                Box::new(WorkspaceDot::new(id))
            });
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
        let corner_r = (ctx.surface_h - 18.0) / 2.0;
        let style = RectStyle {
            fill: Color { r: 0.085, g: 0.095, b: 0.110, a: 1.0 },
            fill_mode: FillMode::Solid,
            corners: Corners {
                tl: CornerShape::Convex,
                tr: CornerShape::Concave,
                br: CornerShape::Convex,
                bl: CornerShape::Concave,
            },
            radius: Corners { tl: 0.0, tr: corner_r, br: corner_r, bl: self.bottom_offset },
            logical_inset: LogicalInset { right: corner_r, bottom: self.bottom_offset, ..Default::default() },
            ..Default::default()
        };
        surface.draw_rect(
            ctx.surface_w,
            ctx.surface_h,
            self.panel_width.value(),
            ctx.surface_h,
            &style,
            Mat3::identity(),
        );

        self.padded_row.draw(surface, ctx);
    }

    fn on_click(&self, x: f32, y: f32, ctx: &RenderContext) -> bool {
        self.padded_row.on_click(x, y, ctx)
    }
}
