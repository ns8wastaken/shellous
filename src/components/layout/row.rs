use crate::components::canvas::{Size, stack_horizontal};
use crate::components::canvas::Rect;
use crate::renderer::batch::DrawBatch;
use crate::services::workspace::WorkspaceSnapshot;
use crate::components::ui::{Element, RenderContext};

// ==================== ROW ====================

pub struct Row {
    pub children: Vec<Box<dyn Element>>,
    spacing: f32,
}

impl Row {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            spacing: 8.0,
        }
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
}

impl Row {
    fn child_layouts(&self, available: Size) -> Vec<Size> {
        self.children.iter().map(|c| c.layout(available)).collect()
    }
}

impl Element for Row {
    fn update(&mut self, snapshot: &WorkspaceSnapshot) {
        for child in &mut self.children {
            child.update(snapshot);
        }
    }

    fn tick_animations(&mut self, absolute_time: f32) -> bool {
        let mut active = false;
        for child in &mut self.children {
            if child.tick_animations(absolute_time) {
                active = true;
            }
        }
        active
    }

    fn layout(&self, available: Size) -> Size {
        let sizes = self.child_layouts(available);
        let total_w: f32 = sizes.iter().map(|s| s.w).sum();
        let spacing_w = if sizes.len() > 1 {
            (sizes.len() - 1) as f32 * self.spacing
        } else {
            0.0
        };
        let max_h = sizes.iter().map(|s| s.h).fold(0.0f32, f32::max);
        Size { w: total_w + spacing_w, h: max_h }
    }

    fn draw(&self, rect: Rect, batch: &mut DrawBatch, ctx: &RenderContext) {
        let sizes = self.child_layouts(rect.size());
        let child_rects = stack_horizontal(rect, &sizes, self.spacing);
        for (child, child_rect) in self.children.iter().zip(child_rects) {
            child.draw(child_rect, batch, ctx);
        }
    }

    fn on_click(&self, rect: Rect, x: f32, y: f32, ctx: &RenderContext) -> bool {
        let sizes = self.child_layouts(rect.size());
        let child_rects = stack_horizontal(rect, &sizes, self.spacing);
        for (child, child_rect) in self.children.iter().zip(child_rects).rev() {
            if child_rect.contains(x, y) && child.on_click(child_rect, x, y, ctx) {
                return true;
            }
        }
        false
    }
}
