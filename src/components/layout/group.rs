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

    fn draw(&self, rect: Rect, batch: &mut DrawBatch, ctx: &RenderContext) {
        for child in &self.children {
            child.draw(rect, batch, ctx);
        }
    }

    fn on_click(&self, rect: Rect, x: f32, y: f32, ctx: &RenderContext) -> bool {
        for child in self.children.iter().rev() {
            if child.on_click(rect, x, y, ctx) {
                return true;
            }
        }
        false
    }
}
