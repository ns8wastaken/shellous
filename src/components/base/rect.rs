use crate::components::arena::Slot;
use crate::components::base::Element;
use crate::components::geom::{Rect, Size};
use crate::components::layout_tree::LayoutNode;
use crate::components::ui::{ElementArena, RenderContext};
use crate::renderer::animation::cache::AnimationCache;
use crate::renderer::batch::{DrawBatch, DrawParams};
use crate::renderer::programs::rect::RectStyle;

pub struct RectNode {
    pub style: RectStyle,
    pub size: Size,
    pub on_click: Option<Box<dyn Fn()>>,
}

impl RectNode {
    pub fn new(size: Size, style: RectStyle) -> Self {
        Self { style, size, on_click: None }
    }

    pub fn with_click(mut self, f: Box<dyn Fn()>) -> Self {
        self.on_click = Some(f);
        self
    }
}

impl Element for RectNode {
    fn children(&self) -> &[Slot] { &[] }

    fn layout(&self, _available: Size, _cache: &AnimationCache, _arena: &ElementArena) -> Size {
        self.size
    }

    fn layout_tree(&self, rect: Rect, _cache: &AnimationCache, _arena: &ElementArena) -> LayoutNode {
        LayoutNode::new(rect)
    }

    fn draw(&self, node: &LayoutNode, batch: &mut DrawBatch, _ctx: &RenderContext) {
        batch.push(node.rect, DrawParams::Rect(self.style.clone()));
    }

    fn on_click(&self, node: &LayoutNode, x: f32, y: f32, _ctx: &RenderContext) -> bool {
        if node.rect.contains(x, y) {
            if let Some(f) = &self.on_click {
                f();
            }
            true
        } else {
            false
        }
    }
}
