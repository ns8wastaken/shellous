use crate::canvas::Canvas;
use crate::renderer::programs::rect::{
    Color, CornerShape, Corners, FillMode, LogicalInset, Mat3, RectStyle,
};
use crate::ui::{Element, RenderContext};

pub struct MiddlePanel {
    pub width: f32,
}

impl Default for MiddlePanel {
    fn default() -> Self {
        Self { width: 260.0 }
    }
}

impl Element for MiddlePanel {
    fn draw(&self, canvas: &Canvas, ctx: &RenderContext) {
        let panel_h = ctx.surface_h - 18.0;
        let x = ((ctx.surface_w - self.width) * 0.5).max(0.0);
        draw_background(canvas, ctx.surface_w, ctx.surface_h, panel_h, self.width, x);
    }
}

fn draw_background(
    canvas: &Canvas,
    surface_w: f32,
    surface_h: f32,
    panel_h: f32,
    panel_w: f32,
    x: f32,
) {
    let style = RectStyle {
        fill: Color { r: 0.085, g: 0.095, b: 0.110, a: 1.0 },
        fill_mode: FillMode::Solid,
        corners: Corners {
            tl: CornerShape::Concave,
            tr: CornerShape::Concave,
            br: CornerShape::Convex,
            bl: CornerShape::Convex,
        },
        radius: Corners { tl: 12.0, tr: 12.0, br: 12.0, bl: 12.0 },
        logical_inset: LogicalInset { right: 12.0, left: 12.0, ..Default::default() },
        ..Default::default()
    };
    canvas.draw_rect(
        surface_w,
        surface_h,
        panel_w,
        panel_h,
        &style,
        Mat3::translation(x, 0.0)
    );
}
