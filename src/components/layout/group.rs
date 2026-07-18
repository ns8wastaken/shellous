use crate::components::layout_tree::LayoutNode;
use crate::components::rect::{Rect, Size};
use crate::renderer::animation::cache::AnimationCache;
use crate::renderer::batch::DrawBatch;
use crate::components::ui::{Element, RenderContext};
use crate::shell::event::ShellEvent;

// ==================== GROUP ====================

/// A container that holds child elements and delegates to each sequentially.
/// Children receive the same rect — Group applies no layout constraint.
pub struct Group {
    pub children: Vec<Box<dyn Element>>,
}

impl Group {
    pub fn new(children: Vec<Box<dyn Element>>) -> Self {
        Self { children }
    }
}

impl Element for Group {
    fn update(&mut self, event: &ShellEvent, now: f32, cache: &mut AnimationCache) -> bool {
        let mut changed = false;
        for child in &mut self.children {
            changed |= child.update(event, now, cache);
        }
        changed
    }

    fn derive_targets(&self, now: f32, cache: &mut AnimationCache) {
        for child in &self.children {
            child.derive_targets(now, cache);
        }
    }

    fn layout(&self, available: Size, cache: &AnimationCache) -> Size {
        // Group takes all available space
        let _ = cache;
        available
    }

    fn layout_tree(&self, rect: Rect, cache: &AnimationCache) -> LayoutNode {
        LayoutNode {
            rect,
            children: self.children.iter().map(|c| c.layout_tree(rect, cache)).collect(),
        }
    }

    fn draw(&self, node: &LayoutNode, batch: &mut DrawBatch, ctx: &RenderContext) {
        for (child, child_node) in self.children.iter().zip(&node.children) {
            child.draw(child_node, batch, ctx);
        }
    }

    fn on_click(&self, node: &LayoutNode, x: f32, y: f32, ctx: &RenderContext) -> bool {
        for (child, child_node) in self.children.iter().zip(&node.children).rev() {
            if child_node.rect.contains(x, y) && child.on_click(child_node, x, y, ctx) {
                return true;
            }
        }
        false
    }
}
