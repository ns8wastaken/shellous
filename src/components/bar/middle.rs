use crate::components::canvas::DrawingSurface;
use crate::components::ui::{Element, RenderContext};
use crate::renderer::programs::rect::{
    CornerShape, Mat3, RectStyle,
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
    fn draw(&self, surface: &dyn DrawingSurface, ctx: &RenderContext) {
        let panel_h = ctx.surface_h - (BAR_HEIGHT - CORNER_RADIUS);
        let x = ((ctx.surface_w - self.width) * 0.5).max(0.0);
        draw_background(surface, panel_h, self.width, x);
    }
}

fn draw_background(
    surface: &dyn DrawingSurface,
    panel_h: f32,
    panel_w: f32,
    x: f32,
) {
    surface.draw_rect(
        panel_w,
        panel_h,
        &RectStyle::new()
            .fill(0.085, 0.095, 0.110, 1.0)
            .corners(
                CornerShape::Concave,
                CornerShape::Concave,
                CornerShape::Convex,
                CornerShape::Convex,
            )
            .all_radius(CORNER_RADIUS)
            .inset_left(CORNER_RADIUS)
            .inset_right(CORNER_RADIUS),
        Mat3::translation(x, 0.0),
    );
}
