use crate::components::layout::stack_horizontal;
use crate::components::rect::{Rect, Size};
use crate::components::keyed_list::KeyedList;
use crate::components::ui::{Element, RenderContext};
use crate::renderer::animation::Animated;
use crate::renderer::animation::easing::Easing;
use crate::renderer::batch::DrawBatch;
use crate::renderer::programs::rect::{
    CornerShape, RectStyle,
};
use crate::services::workspace::WorkspaceSnapshot;
use super::{BAR_HEIGHT, workspace_dot::{WorkspaceDot, WORKSPACE_R}};

const WORKSPACE_SPACING: f32 = 8.0;

const LEFT_PAD: f32 = BAR_HEIGHT / 2.0;
const RIGHT_PAD: f32 = BAR_HEIGHT;
const TOP: f32 = BAR_HEIGHT / 2.0 - WORKSPACE_R;

// ==================== LEFT PANEL ====================

pub struct LeftPanel {
    dots: KeyedList<i32, WorkspaceDot>,
    panel_width: Animated<f32>,
    bottom_offset: f32,
}

impl LeftPanel {
    pub fn new(bottom_offset: f32) -> Self {
        Self {
            dots: KeyedList::new(),
            panel_width: Animated::new(LEFT_PAD + RIGHT_PAD)
                .with_duration(0.26)
                .with_easing(Easing::EaseOutCubic),
            bottom_offset,
        }
    }

    fn dot_row_width(&self, available: Size) -> f32 {
        let n = self.dots.len();
        if n == 0 {
            return 0.0;
        }
        let sizes: Vec<Size> = self.dots.iter().map(|d| d.layout(available)).collect();
        sizes.iter().map(|s| s.w).sum::<f32>()
            + (n as f32 - 1.0) * WORKSPACE_SPACING
    }
}

impl Element for LeftPanel {
    fn update(&mut self, snapshot: &WorkspaceSnapshot) {
        let mut cur_ids: Vec<i32> = snapshot.workspaces.iter().map(|w| w.id).collect();
        cur_ids.sort_unstable();
        self.dots.reconcile(&cur_ids, |id| WorkspaceDot::new(id));
        for dot in self.dots.iter_mut() {
            dot.update(snapshot);
        }
    }

    fn tick_animations(&mut self, absolute_time: f32) -> bool {
        let mut any = false;
        for dot in self.dots.iter_mut() {
            if dot.tick_animations(absolute_time) {
                any = true;
            }
        }
        let dot_size = Size::new(0.0, 0.0);
        let target = LEFT_PAD + self.dot_row_width(dot_size) + RIGHT_PAD;
        if target != self.panel_width.target() {
            self.panel_width.set_target(target, absolute_time);
        }
        let panel_animating = self.panel_width.tick(absolute_time);
        any || panel_animating
    }

    fn layout(&self, _available: Size) -> Size {
        Size { w: self.panel_width.value(), h: BAR_HEIGHT }
    }

    fn draw(&self, rect: Rect, batch: &mut DrawBatch, ctx: &RenderContext) {
        let corner_r = (ctx.surface_h - self.bottom_offset) / 2.0;
        let w = self.panel_width.value();
        let h = ctx.surface_h;

        let bg_rect = Rect { w, h, ..rect };
        let base_style = RectStyle::new()
            .corners(
                CornerShape::Convex,
                CornerShape::Concave,
                CornerShape::Convex,
                CornerShape::Concave,
            )
            .radius(0.0, corner_r, corner_r, self.bottom_offset)
            .inset_right(corner_r)
            .inset_bottom(self.bottom_offset);

        // Shadow pass
        batch.push(
            bg_rect,
            &base_style
                .clone()
                .fill(0.0, 0.0, 0.0, 0.5)
                .softness(20.0)
                .shadow(0.0, 0.0),
        );

        // Fill pass
        batch.push(
            bg_rect,
            &base_style
                .clone()
                .fill(0.085, 0.095, 0.110, 1.0),
        );

        // Dot row
        let content = rect.inset(LEFT_PAD, TOP, 0.0, 0.0);
        let dot_sizes: Vec<Size> = self.dots.iter().map(|d| d.layout(rect.size())).collect();
        let dot_rects = stack_horizontal(content, &dot_sizes, WORKSPACE_SPACING);
        for (dot, dot_rect) in self.dots.iter().zip(dot_rects) {
            dot.draw(dot_rect, batch, ctx);
        }
    }

    fn on_click(&self, rect: Rect, x: f32, y: f32, ctx: &RenderContext) -> bool {
        let dot_sizes: Vec<Size> = self.dots.iter().map(|d| d.layout(rect.size())).collect();
        let content = rect.inset(LEFT_PAD, TOP, 0.0, 0.0);
        let dot_rects = stack_horizontal(content, &dot_sizes, WORKSPACE_SPACING);
        for (dot, dot_rect) in self.dots.iter().zip(dot_rects) {
            if dot_rect.contains(x, y) && dot.on_click(dot_rect, x, y, ctx) {
                return true;
            }
        }
        false
    }
}
