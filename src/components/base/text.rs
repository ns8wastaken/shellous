use crate::components::arena::Slot;
use crate::components::base::Element;
use crate::components::geom::{Rect, Size};
use crate::components::layout_tree::LayoutNode;
use crate::components::ui::{ElementArena, RenderContext};
use crate::renderer::animation::cache::AnimationCache;
use crate::renderer::batch::{DrawBatch, DrawParams};
use crate::renderer::programs::text::TextStyle;
use crate::renderer::types::Color;

pub struct TextNode {
    pub text: String,
    pub font_size: f32,
    pub color: Color,
}

impl TextNode {
    pub fn new(text: impl Into<String>, font_size: f32, color: Color) -> Self {
        Self { text: text.into(), font_size, color }
    }
}

impl Element for TextNode {
    fn children(&self) -> &[Slot] { &[] }

    fn layout(&self, _available: Size, _cache: &AnimationCache, _arena: &ElementArena) -> Size {
        // ponytail: font measurement stubbed; returns a fixed size.
        // Proper measurement will use fontdue::Font in Phase 6.
        let w = self.text.len() as f32 * self.font_size * 0.6;
        Size::new(w, self.font_size * 1.2)
    }

    fn layout_tree(&self, rect: Rect, _cache: &AnimationCache, _arena: &ElementArena) -> LayoutNode {
        LayoutNode::new(rect)
    }

    fn draw(&self, node: &LayoutNode, batch: &mut DrawBatch, _ctx: &RenderContext) {
        batch.push(
            node.rect,
            DrawParams::Text(
                // TODO: make the style just be size and color
                TextStyle::new()
                    .text(self.text.clone())
                    .size(self.font_size)
                    .color(self.color),
            ),
        );
    }

    fn on_click(&self, _node: &LayoutNode, _x: f32, _y: f32, _ctx: &RenderContext) -> bool {
        false
    }
}
