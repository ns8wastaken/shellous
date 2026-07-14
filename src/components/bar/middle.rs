use crate::components::layout_tree::LayoutNode;
use crate::components::rect::Size;
use crate::components::ui::{Element, RenderContext};
use crate::renderer::batch::{DrawBatch, DrawParams};
use crate::renderer::programs::rect::{
    CornerShape, RectStyle,
};
use crate::renderer::programs::text::TextStyle;
use crate::renderer::types::Color;
use super::{BAR_HEIGHT, CORNER_RADIUS};

pub struct MiddlePanel {
    pub width: f32,
}

impl Default for MiddlePanel {
    fn default() -> Self {
        Self { width: 260.0 }
    }
}

impl Element for MiddlePanel {
    fn layout(&self, _available: Size) -> Size {
        Size { w: self.width, h: BAR_HEIGHT }
    }

    fn draw(&self, node: &LayoutNode, batch: &mut DrawBatch, _ctx: &RenderContext) {
        let base_style = RectStyle::new()
            .corners(
                CornerShape::Concave,
                CornerShape::Concave,
                CornerShape::Convex,
                CornerShape::Convex,
            )
            .all_radius(CORNER_RADIUS)
            .inset_left(CORNER_RADIUS)
            .inset_right(CORNER_RADIUS);

        batch.push(
            node.rect,
            DrawParams::Rect(
                base_style
                    .clone()
                    .fill(0.0, 0.0, 0.0, 0.5)
                    .softness(10.0)
                    .shadow(0.0, 0.0)
            )
        );

        batch.push(
            node.rect,
            DrawParams::Rect(base_style.fill(0.085, 0.095, 0.110, 1.0)),
        );

        let mut text_rect = node.rect;
        text_rect.x += 20.0; // Left padding offset
        text_rect.y += 10.0; // Top padding offset matching your design bounds

        batch.push(
            text_rect,
            DrawParams::Text(
                TextStyle::new()
                    .text("Hello World")
                    .size(12.0)
                    .color(Color::rgb(1.0, 1.0, 1.0))
            )
        );
    }
}
