use crate::components::canvas::{DrawingSurface, TranslatedCanvas};
use crate::components::keyed_list::KeyedList;
use crate::components::ui::{Element, RenderContext};
use crate::renderer::animation::Animated;
use crate::renderer::animation::easing::Easing;
use crate::renderer::programs::rect::{
    CornerShape, Mat3, RectStyle,
};
use crate::services::workspace::WorkspaceSnapshot;
use super::{BAR_HEIGHT, workspace_dot::{WorkspaceDot, WORKSPACE_R}};

const WORKSPACE_SPACING: f32 = 8.0;

// Layout constants mirroring the old Padding+Row setup.
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

    fn dot_row_width(&self) -> f32 {
        let n = self.dots.len();
        if n == 0 {
            return 0.0;
        }
        self.dots.iter().map(|d| d.size().0).sum::<f32>()
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
        let target = LEFT_PAD + self.dot_row_width() + RIGHT_PAD;
        if target != self.panel_width.target() {
            self.panel_width.set_target(target, absolute_time);
        }
        let panel_animating = self.panel_width.tick(absolute_time);
        any || panel_animating
    }

    fn draw(&self, surface: &dyn DrawingSurface, ctx: &RenderContext) {
        let corner_r = (ctx.surface_h - self.bottom_offset) / 2.0;
        let w = self.panel_width.value();
        let h = ctx.surface_h;

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
        surface.draw_rect(
            w, h,
            &base_style
                .clone()
                .fill(0.0, 0.0, 0.0, 1.0)
                .softness(20.0)
                .shadow(0.0, 0.0),
            Mat3::identity(),
        );

        // Fill pass
        surface.draw_rect(
            w, h,
            &base_style
                .clone()
                .fill(0.085, 0.095, 0.110, 1.0),
            Mat3::identity(),
        );

        let mut cx = LEFT_PAD;
        for dot in self.dots.iter() {
            let tc = TranslatedCanvas::new(surface, cx, TOP);
            dot.draw(&tc, ctx);
            cx += dot.size().0 + WORKSPACE_SPACING;
        }
    }

    fn on_click(&self, x: f32, y: f32, ctx: &RenderContext) -> bool {
        let mut cx = LEFT_PAD;
        for dot in self.dots.iter() {
            let (dw, dh) = dot.size();
            if x >= cx && x <= cx + dw && y >= TOP && y <= TOP + dh {
                if dot.on_click(x - cx, y - TOP, ctx) {
                    return true;
                }
            }
            cx += dw + WORKSPACE_SPACING;
        }
        false
    }

    fn size(&self) -> (f32, f32) {
        (self.panel_width.value(), BAR_HEIGHT)
    }
}
