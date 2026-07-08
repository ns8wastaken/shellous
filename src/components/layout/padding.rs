use crate::components::canvas::{Rect, Size};
use crate::renderer::batch::DrawBatch;
use crate::services::workspace::WorkspaceSnapshot;
use crate::components::ui::{Element, RenderContext};

// ==================== PADDING ====================

/// A container that insets a single child element by `left`/`top`/`right`/`bottom`.
pub struct Padding {
    pub child: Box<dyn Element>,
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl Padding {
    pub fn new(child: Box<dyn Element>) -> Self {
        Self {
            child,
            left: 0.0,
            right: 0.0,
            top: 0.0,
            bottom: 0.0,
        }
    }

    pub fn left(mut self, v: f32) -> Self {
        self.left = v;
        self
    }

    pub fn right(mut self, v: f32) -> Self {
        self.right = v;
        self
    }

    pub fn top(mut self, v: f32) -> Self {
        self.top = v;
        self
    }

    pub fn bottom(mut self, v: f32) -> Self {
        self.bottom = v;
        self
    }

    pub fn all(mut self, v: f32) -> Self {
        self.left = v;
        self.right = v;
        self.top = v;
        self.bottom = v;
        self
    }

    pub fn x(mut self, v: f32) -> Self {
        self.left = v;
        self.right = v;
        self
    }

    pub fn y(mut self, v: f32) -> Self {
        self.top = v;
        self.bottom = v;
        self
    }
}

impl Element for Padding {
    fn update(&mut self, snapshot: &WorkspaceSnapshot) {
        self.child.update(snapshot);
    }

    fn tick_animations(&mut self, absolute_time: f32) -> bool {
        self.child.tick_animations(absolute_time)
    }

    fn layout(&self, available: Size) -> Size {
        let inner = Size {
            w: (available.w - self.left - self.right).max(0.0),
            h: (available.h - self.top - self.bottom).max(0.0),
        };
        let child_size = self.child.layout(inner);
        Size {
            w: child_size.w + self.left + self.right,
            h: child_size.h + self.top + self.bottom,
        }
    }

    fn draw(&self, rect: Rect, batch: &mut DrawBatch, ctx: &RenderContext) {
        let child_rect = rect.inset(self.left, self.top, self.right, self.bottom);
        self.child.draw(child_rect, batch, ctx);
    }

    fn on_click(&self, rect: Rect, x: f32, y: f32, ctx: &RenderContext) -> bool {
        let child_rect = rect.inset(self.left, self.top, self.right, self.bottom);
        self.child.on_click(child_rect, x, y, ctx)
    }
}
