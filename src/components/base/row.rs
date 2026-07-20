use crate::components::arena::Slot;
use crate::components::base::{Element, stack_horizontal};
use crate::components::geom::{Rect, Size};
use crate::components::layout_tree::LayoutNode;
use crate::components::ui::{ElementArena, RenderContext};
use crate::renderer::animation::cache::AnimationCache;
use crate::renderer::batch::DrawBatch;

pub struct RowNode {
    pub children: Vec<Slot>,
    pub spacing: f32,
}

impl RowNode {
    pub fn new(children: Vec<Slot>, spacing: f32) -> Self {
        Self { children, spacing }
    }
}

impl Element for RowNode {
    fn children(&self) -> &[Slot] { &self.children }

    fn layout(&self, available: Size, cache: &AnimationCache, arena: &ElementArena) -> Size {
        let mut total_w = 0.0f32;
        let mut max_h = 0.0f32;
        for &slot in &self.children {
            let child = arena.get(slot).unwrap();
            let size = child.layout(available, cache, arena);
            total_w += size.w;
            if size.h > max_h { max_h = size.h; }
        }
        if !self.children.is_empty() {
            total_w += self.spacing * (self.children.len() - 1) as f32;
        }
        Size::new(total_w, max_h)
    }

    fn layout_tree(&self, rect: Rect, cache: &AnimationCache, arena: &ElementArena) -> LayoutNode {
        let child_sizes: Vec<Size> = self.children.iter()
            .map(|&slot| arena.get(slot).unwrap().layout(rect.size(), cache, arena))
            .collect();
        let child_rects = stack_horizontal(rect, &child_sizes, self.spacing);
        LayoutNode {
            rect,
            children: child_rects.iter().zip(&self.children)
                .map(|(&child_rect, &slot)| {
                    arena.get(slot).unwrap().layout_tree(child_rect, cache, arena)
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
