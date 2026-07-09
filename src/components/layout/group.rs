use crate::components::layout_tree::LayoutNode;
use crate::components::rect::{Rect, Size};
use crate::renderer::batch::DrawBatch;
use crate::services::workspace::WorkspaceSnapshot;
use crate::components::ui::{Element, RenderContext};

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
        // Group takes all available space
        available
    }

    fn layout_tree(&self, rect: Rect) -> LayoutNode {
        LayoutNode {
            rect,
            children: self.children.iter().map(|c| c.layout_tree(rect)).collect(),
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
