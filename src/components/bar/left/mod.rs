mod workspace_dot;

use crate::components::layout::stack_horizontal;
use crate::components::layout_tree::LayoutNode;
use crate::components::rect::{Rect, Size};
use crate::components::keyed_list::KeyedList;
use crate::components::ui::{Element, RenderContext};
use crate::renderer::animation::cache::{AnimationCache, AnimSlot, AnimSpec};
use crate::renderer::animation::easing::Easing;
use crate::renderer::batch::{DrawBatch, DrawParams};
use crate::renderer::programs::rect::{
    CornerShape, RectStyle,
};
use crate::shell::event::ShellEvent;
use crate::components::bar::BAR_HEIGHT;
use workspace_dot::{WorkspaceDot, WORKSPACE_R};

const WORKSPACE_SPACING: f32 = 8.0;

const LEFT_PAD: f32 = BAR_HEIGHT / 2.0;
const RIGHT_PAD: f32 = BAR_HEIGHT;
const TOP: f32 = BAR_HEIGHT / 2.0 - WORKSPACE_R;

pub struct LeftPanel {
    dots: KeyedList<i32, WorkspaceDot>,
    panel_width: AnimSlot,
    bottom_offset: f32,
}

impl LeftPanel {
    pub fn new(bottom_offset: f32, cache: &mut AnimationCache) -> Self {
        Self {
            dots: KeyedList::new(),
            panel_width: cache.insert(
                AnimSpec::new(LEFT_PAD + RIGHT_PAD)
                    .with_duration(0.26)
                    .with_easing(Easing::EaseOutCubic),
            ),
            bottom_offset,
        }
    }

    fn dot_row_value(&self, cache: &AnimationCache) -> f32 {
        let n = self.dots.len();
        if n == 0 {
            return 0.0;
        }
        let values: Vec<f32> = self.dots.iter().map(|d| cache.value(d.width)).collect();
        values.iter().sum::<f32>()
            + (n as f32 - 1.0) * WORKSPACE_SPACING
    }
}

impl Element for LeftPanel {
    fn update(&mut self, event: &ShellEvent, now: f32, cache: &mut AnimationCache) -> bool {
        if let ShellEvent::WorkspaceUpdated(snapshot) = event {
            let mut cur_ids: Vec<i32> = snapshot.workspaces.iter().map(|w| w.id).collect();
            cur_ids.sort_unstable();
            self.dots.reconcile(&cur_ids, |id| WorkspaceDot::new(id, cache));
            for dot in self.dots.iter_mut() {
                dot.update(event, now, cache);
            }
            let target = LEFT_PAD + self.dot_row_value(cache) + RIGHT_PAD;
            cache.set_target(self.panel_width, target, now);
            true
        } else {
            false
        }
    }

    fn derive_targets(&self, now: f32, cache: &mut AnimationCache) {
        let target = LEFT_PAD + self.dot_row_value(cache) + RIGHT_PAD;
        if target != cache.target(self.panel_width) {
            cache.set_target(self.panel_width, target, now);
        }
    }

    fn layout(&self, _available: Size, cache: &AnimationCache) -> Size {
        Size { w: cache.value(self.panel_width), h: BAR_HEIGHT }
    }

    fn layout_tree(&self, rect: Rect, cache: &AnimationCache) -> LayoutNode {
        let content = rect.inset(LEFT_PAD, TOP, 0.0, 0.0);
        let dot_sizes: Vec<Size> = self.dots
            .iter()
            .map(|d| d.layout(rect.size(), cache))
            .collect();
        let dot_rects = stack_horizontal(content, &dot_sizes, WORKSPACE_SPACING);
        LayoutNode {
            rect,
            children: self.dots
                .iter()
                .zip(dot_rects)
                .map(|(d, r)| d.layout_tree(r, cache))
                .collect(),
        }
    }

    fn draw(&self, node: &LayoutNode, batch: &mut DrawBatch, ctx: &RenderContext) {
        let corner_r = (ctx.surface_h - self.bottom_offset) / 2.0;
        let w = ctx.animations.value(self.panel_width);
        let h = ctx.surface_h;

        let bg_rect = Rect { w, h, ..node.rect };
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
            DrawParams::Rect(
                base_style
                    .clone()
                    .fill(0.0, 0.0, 0.0, 0.5)
                    .softness(10.0)
                    .shadow(0.0, 0.0)
            )
        );

        // Fill pass
        batch.push(
            bg_rect,
            DrawParams::Rect(base_style.fill(0.085, 0.095, 0.110, 1.0)),
        );

        // Dot row
        for (dot, child_node) in self.dots.iter().zip(&node.children) {
            dot.draw(child_node, batch, ctx);
        }
    }

    fn on_click(&self, node: &LayoutNode, x: f32, y: f32, ctx: &RenderContext) -> bool {
        for i in (0..self.dots.items.len()).rev() {
            let (_, dot) = &self.dots.items[i];
            let child_node = &node.children[i];
            if child_node.rect.contains(x, y) && dot.on_click(child_node, x, y, ctx) {
                return true;
            }
        }
        false
    }
}
