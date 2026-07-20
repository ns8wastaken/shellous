use crate::components::arena::Slot;
use crate::components::base::Element;
use crate::components::geom::{Rect, Size};
use crate::components::layout_tree::LayoutNode;
use crate::components::ui::{ElementArena, RenderContext};
use crate::renderer::animation::cache::AnimationCache;
use crate::renderer::batch::DrawBatch;

// ponytail: Group stores children as Vec<Slot> — children live in the per-surface
// arena, not as owned trait objects. Nodes are resolved through the arena during
// layout/draw/click passes.

pub struct GroupNode {
    pub children: Vec<Slot>,
}

impl GroupNode {
    pub fn new(children: Vec<Slot>) -> Self {
        Self { children }
    }
}

impl Element for GroupNode {
    fn children(&self) -> &[Slot] { &self.children }

    fn layout(&self, available: Size, cache: &AnimationCache, arena: &ElementArena) -> Size {
        let mut mw = 0.0f32;
        let mut mh = 0.0f32;
        for &slot in &self.children {
            let size = arena.get(slot).unwrap().layout(available, cache, arena);
            if size.w > mw { mw = size.w; }
            if size.h > mh { mh = size.h; }
        }
        Size::new(mw, mh)
    }

    fn layout_tree(&self, rect: Rect, cache: &AnimationCache, arena: &ElementArena) -> LayoutNode {
        LayoutNode {
            rect,
            children: self
                .children
                .iter()
                .map(|&slot| {
                    arena
                        .get(slot)
                        .unwrap()
                        .layout_tree(rect, cache, arena)
                })
                .collect(),
        }
    }

    fn draw(&self, node: &LayoutNode, batch: &mut DrawBatch, ctx: &RenderContext) {
        for (&slot, child_node) in self.children.iter().zip(&node.children) {
            ctx.arena.get(slot).unwrap().draw(child_node, batch, ctx);
        }
    }

    fn on_click(&self, node: &LayoutNode, x: f32, y: f32, ctx: &RenderContext) -> bool {
        for (&slot, child_node) in self.children.iter().zip(&node.children).rev() {
            if child_node.rect.contains(x, y)
                && ctx
                    .arena
                    .get(slot)
                    .unwrap()
                    .on_click(child_node, x, y, ctx)
            {
                return true;
            }
        }
        false
    }
}
