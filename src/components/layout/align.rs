use crate::components::canvas::{Alignment, Rect, Size};
use crate::renderer::batch::DrawBatch;
use crate::services::workspace::WorkspaceSnapshot;
use crate::components::ui::{Element, RenderContext};

// ==================== ALIGN ====================

pub struct Align {
    pub child: Box<dyn Element>,
    pub alignment: Alignment,
}

impl Align {
    pub fn new(child: Box<dyn Element>, alignment: Alignment) -> Self {
        Self { child, alignment }
    }
}

impl Element for Align {
    fn update(&mut self, snapshot: &WorkspaceSnapshot) {
        self.child.update(snapshot);
    }

    fn tick_animations(&mut self, absolute_time: f32) -> bool {
        self.child.tick_animations(absolute_time)
    }

    fn layout(&self, available: Size) -> Size {
        self.child.layout(available)
    }

    fn draw(&self, rect: Rect, batch: &mut DrawBatch, ctx: &RenderContext) {
        let child_size = self.child.layout(rect.size());
        let child_rect = self.alignment.apply(rect, child_size);
        self.child.draw(child_rect, batch, ctx);
    }

    fn on_click(&self, rect: Rect, x: f32, y: f32, ctx: &RenderContext) -> bool {
        let child_size = self.child.layout(rect.size());
        let child_rect = self.alignment.apply(rect, child_size);
        self.child.on_click(child_rect, x, y, ctx)
    }

    fn id(&self) -> Option<i32> {
        self.child.id()
    }
}
