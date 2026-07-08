use crate::components::canvas::{Rect, Size, align_top_center};
use crate::components::ui::{Element, RenderContext};
use crate::renderer::batch::DrawBatch;
use crate::renderer::programs::rect::{
    CornerShape, RectStyle,
};
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

    fn draw(&self, rect: Rect, batch: &mut DrawBatch, _ctx: &RenderContext) {
        let panel_h = BAR_HEIGHT;
        let (x, y) = align_top_center(rect, Size::new(self.width, panel_h));
        let bg_rect = Rect::new(x, y, self.width, panel_h);
        draw_background(bg_rect, batch);
    }
}

fn draw_background(
    rect: Rect,
    batch: &mut DrawBatch,
) {
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
        rect,
        &base_style
            .clone()
            .fill(0.0, 0.0, 0.0, 0.5)
            .softness(20.0)
            .shadow(0.0, 0.0),
    );

    batch.push(
        rect,
        &base_style.fill(0.085, 0.095, 0.110, 1.0),
    );
}
