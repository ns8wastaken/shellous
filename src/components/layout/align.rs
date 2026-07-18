use crate::components::layout::Alignment;
use crate::components::layout_tree::LayoutNode;
use crate::components::rect::{Rect, Size};
use crate::renderer::animation::cache::AnimationCache;
use crate::renderer::batch::DrawBatch;
use crate::components::ui::{Element, RenderContext};
use crate::shell::event::ShellEvent;

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
    fn update(&mut self, event: &ShellEvent, now: f32, cache: &mut AnimationCache) -> bool {
        self.child.update(event, now, cache)
    }

    fn derive_targets(&self, now: f32, cache: &mut AnimationCache) {
        self.child.derive_targets(now, cache);
    }

    fn layout(&self, available: Size, cache: &AnimationCache) -> Size {
        self.child.layout(available, cache)
    }

    fn layout_tree(&self, rect: Rect, cache: &AnimationCache) -> LayoutNode {
        let child_size = self.child.layout(rect.size(), cache);
        let child_rect = self.alignment.apply(rect, child_size);
        LayoutNode {
            rect,
            children: vec![self.child.layout_tree(child_rect, cache)],
        }
    }

    fn draw(&self, node: &LayoutNode, batch: &mut DrawBatch, ctx: &RenderContext) {
        self.child.draw(&node.children[0], batch, ctx);
    }

    fn on_click(&self, node: &LayoutNode, x: f32, y: f32, ctx: &RenderContext) -> bool {
        self.child.on_click(&node.children[0], x, y, ctx)
    }
}
