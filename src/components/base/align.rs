use crate::components::arena::Slot;
use crate::components::base::{Alignment, Element};
use crate::components::geom::{Rect, Size};
use crate::components::layout_tree::LayoutNode;
use crate::components::ui::{ElementArena, RenderContext};
use crate::renderer::animation::cache::AnimationCache;
use crate::renderer::batch::DrawBatch;

// ponytail: Align stores child as a Slot — the child lives in the per-surface
// arena, not as an owned trait object.

pub struct AlignNode {
    pub child: Slot,
    pub alignment: Alignment,
}

impl AlignNode {
    pub fn new(child: Slot, alignment: Alignment) -> Self {
        Self { child, alignment }
    }
}

impl Element for AlignNode {
    fn children(&self) -> &[Slot] { std::slice::from_ref(&self.child) }

    fn layout(&self, available: Size, cache: &AnimationCache, arena: &ElementArena) -> Size {
        let child = arena.get(self.child).unwrap();
        child.layout(available, cache, arena)
    }

    fn layout_tree(&self, rect: Rect, cache: &AnimationCache, arena: &ElementArena) -> LayoutNode {
        let child = arena.get(self.child).unwrap();
        let child_size = child.layout(rect.size(), cache, arena);
        let child_rect = self.alignment.apply(rect, child_size);
        LayoutNode {
            rect,
            children: vec![child.layout_tree(child_rect, cache, arena)],
        }
    }

    fn draw(&self, node: &LayoutNode, batch: &mut DrawBatch, ctx: &RenderContext) {
        ctx.arena
            .get(self.child)
            .unwrap()
            .draw(&node.children[0], batch, ctx);
    }

    fn on_click(&self, node: &LayoutNode, x: f32, y: f32, ctx: &RenderContext) -> bool {
        ctx
            .arena
            .get(self.child)
            .unwrap()
            .on_click(&node.children[0], x, y, ctx)
    }
}
