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
        let panel_h = BAR_HEIGHT;
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

    // Shadow pass
    surface.draw_rect(
        panel_w, panel_h,
        &base_style
            .clone()
            .fill(0.0, 0.0, 0.0, 1.0)
            .softness(20.0)
            .shadow(0.0, 0.0),
        Mat3::translation(x, 0.0),
    );

    // Fill pass
    surface.draw_rect(
        panel_w, panel_h,
        &base_style.fill(0.085, 0.095, 0.110, 1.0),
        Mat3::translation(x, 0.0),
    );
}
