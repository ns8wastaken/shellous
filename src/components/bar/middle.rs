use crate::components::canvas::DrawingSurface;
use crate::components::ui::{Element, RenderContext};
use crate::renderer::programs::rect::{
    Color, Corners, FillMode, LogicalInset, Mat3, RectStyle,
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
        draw_background(surface, ctx.surface_w, ctx.surface_h, panel_h, self.width, x);
    }
}

fn draw_background(
    surface: &dyn DrawingSurface,
    surface_w: f32,
    surface_h: f32,
    panel_h: f32,
    panel_w: f32,
    x: f32,
) {
    let style = RectStyle {
        fill: Color { r: 0.085, g: 0.095, b: 0.110, a: 1.0 },
        fill_mode: FillMode::Solid,
        logical_inset: LogicalInset { right: CORNER_RADIUS, left: CORNER_RADIUS, ..Default::default() },
        radius: Corners { tl: CORNER_RADIUS, tr: CORNER_RADIUS, br: CORNER_RADIUS, bl: CORNER_RADIUS },
        ..Default::default()
    };
    surface.draw_rect(
        surface_w,
        surface_h,
        panel_w,
        panel_h,
        &style,
        Mat3::translation(x, 0.0),
    );
}
