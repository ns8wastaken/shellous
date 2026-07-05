use crate::components::canvas::DrawingSurface;
use crate::services::workspace::WorkspaceSnapshot;
use crate::components::ui::{Element, RenderContext};

// ==================== GROUP ====================

/// A container that holds child elements and delegates to each sequentially.
/// Children position themselves — Group applies no layout transformation.
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

    fn draw(&self, surface: &dyn DrawingSurface, ctx: &RenderContext) {
        for child in &self.children {
            child.draw(surface, ctx);
        }
    }

    fn on_click(&self, x: f32, y: f32, ctx: &RenderContext) -> bool {
        for child in self.children.iter().rev() {
            if child.on_click(x, y, ctx) {
                return true;
            }
        }
        false
    }

}
