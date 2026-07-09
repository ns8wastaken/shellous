use crate::components::layout::Alignment;
use crate::components::layout_tree::LayoutNode;
use crate::components::rect::{Rect, Size};
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

    fn layout_tree(&self, rect: Rect) -> LayoutNode {
        let child_size = self.child.layout(rect.size());
        let child_rect = self.alignment.apply(rect, child_size);
        LayoutNode {
            rect,
            children: vec![self.child.layout_tree(child_rect)],
        }
    }

    fn draw(&self, node: &LayoutNode, batch: &mut DrawBatch, ctx: &RenderContext) {
        self.child.draw(&node.children[0], batch, ctx);
    }

    fn on_click(&self, node: &LayoutNode, x: f32, y: f32, ctx: &RenderContext) -> bool {
        self.child.on_click(&node.children[0], x, y, ctx)
    }
}
