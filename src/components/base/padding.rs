use crate::components::arena::Slot;
use crate::components::base::Element;
use crate::components::geom::{Rect, Size};
use crate::components::layout_tree::LayoutNode;
use crate::components::ui::{ElementArena, RenderContext};
use crate::renderer::animation::cache::AnimationCache;
use crate::renderer::batch::DrawBatch;

pub struct PaddingNode {
    pub child: Slot,
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl PaddingNode {
    pub fn new(child: Slot, insets: (f32, f32, f32, f32)) -> Self {
        Self {
            child,
            left: insets.0,
            top: insets.1,
            right: insets.2,
            bottom: insets.3,
        }
    }
}

impl Element for PaddingNode {
    fn children(&self) -> &[Slot] { std::slice::from_ref(&self.child) }

    fn layout(&self, available: Size, cache: &AnimationCache, arena: &ElementArena) -> Size {
        let cw = (available.w - self.left - self.right).max(0.0);
        let ch = (available.h - self.top - self.bottom).max(0.0);
        let child = arena.get(self.child).unwrap();
        child.layout(Size::new(cw, ch), cache, arena)
    }

    fn layout_tree(&self, rect: Rect, cache: &AnimationCache, arena: &ElementArena) -> LayoutNode {
        let inner = rect.inset(self.left, self.top, self.right, self.bottom);
        let child = arena.get(self.child).unwrap();
        LayoutNode {
            rect,
            children: vec![child.layout_tree(inner, cache, arena)],
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
