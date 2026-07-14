use crate::renderer::types::Color;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum FillMode {
    #[default]
    None,
    Solid,
    LinearGradient,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum GradientDirection {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GradientStop {
    pub position: f32,
    pub color: Color,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum CornerShape {
    #[default]
    Convex,
    Concave,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Corners<T> {
    pub tl: T,
    pub tr: T,
    pub br: T,
    pub bl: T,
}

impl<T: Clone> Corners<T> {
    pub fn all(v: T) -> Self {
        Self { tl: v.clone(), tr: v.clone(), br: v.clone(), bl: v }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LogicalInset {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

#[derive(Clone, Debug, Default)]
pub struct RectStyle {
    pub fill: Color,
    pub border: Color,
    pub fill_mode: FillMode,
    pub gradient_direction: GradientDirection,
    pub gradient_stops: [GradientStop; 4],
    pub corners: Corners<CornerShape>,
    pub logical_inset: LogicalInset,
    pub radius: Corners<f32>,
    pub softness: f32,
    pub no_aa: bool,
    pub invert_fill: bool,
    pub border_width: f32,
    pub outer_shadow: bool,
    pub shadow_cutout_offset_x: f32,
    pub shadow_cutout_offset_y: f32,
}

impl RectStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn solid(fill: Color, radius: f32) -> Self {
        Self {
            fill,
            fill_mode: FillMode::Solid,
            radius: Corners::all(radius),
            ..Default::default()
        }
    }

    pub fn fill(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.fill = Color { r, g, b, a };
        self.fill_mode = FillMode::Solid;
        self
    }

    pub fn fill_color(mut self, color: Color) -> Self {
        self.fill = color;
        self.fill_mode = FillMode::Solid;
        self
    }

    pub fn border(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.border = Color { r, g, b, a };
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border = color;
        self
    }

    pub fn border_width(mut self, w: f32) -> Self {
        self.border_width = w;
        self
    }

    pub fn corners(
        mut self,
        tl: CornerShape,
        tr: CornerShape,
        br: CornerShape,
        bl: CornerShape,
    ) -> Self {
        self.corners = Corners { tl, tr, br, bl };
        self
    }

    pub fn radius(mut self, tl: f32, tr: f32, br: f32, bl: f32) -> Self {
        self.radius = Corners { tl, tr, br, bl };
        self
    }

    pub fn all_radius(mut self, r: f32) -> Self {
        self.radius = Corners::all(r);
        self
    }

    pub fn inset(mut self, l: f32, t: f32, r: f32, b: f32) -> Self {
        self.logical_inset = LogicalInset { left: l, top: t, right: r, bottom: b };
        self
    }

    pub fn inset_left(mut self, v: f32) -> Self {
        self.logical_inset.left = v;
        self
    }

    pub fn inset_top(mut self, v: f32) -> Self {
        self.logical_inset.top = v;
        self
    }

    pub fn inset_right(mut self, v: f32) -> Self {
        self.logical_inset.right = v;
        self
    }

    pub fn inset_bottom(mut self, v: f32) -> Self {
        self.logical_inset.bottom = v;
        self
    }

    pub fn shadow(mut self, dx: f32, dy: f32) -> Self {
        self.outer_shadow = true;
        self.shadow_cutout_offset_x = dx;
        self.shadow_cutout_offset_y = dy;
        self
    }

    pub fn softness(mut self, s: f32) -> Self {
        self.softness = s;
        self
    }
}
