use crate::shell::action::Action;
use crate::renderer::programs::rect::{
    Color, Corners, FillMode, Mat3, RectProgram, RectStyle,
};
use crate::shell::state::ShellState;
use crate::shell::surface_id::SurfaceId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceRole {
    Bar,
    Notifications,
    Launcher,
    Pane,
    Overlay,
}

pub struct RenderContext<'a> {
    pub state: &'a ShellState,
    pub surface_id: SurfaceId,
    pub surface_w: f32,
    pub surface_h: f32,
    pub pointer_pos: Option<(f64, f64)>,
}

pub trait Element {
    fn draw(&self, rect: &RectProgram, ctx: &RenderContext);

    fn on_click(&self, x: f32, y: f32, ctx: &RenderContext) -> Action {
        let _ = (x, y, ctx);
        Action::None
    }
}

pub struct UiTree {
    elements: Vec<Box<dyn Element>>,
}

impl UiTree {
    pub fn new(elements: Vec<Box<dyn Element>>) -> Self {
        Self { elements }
    }

    pub fn draw(&self, rect: &RectProgram, ctx: &RenderContext) {
        for element in &self.elements {
            element.draw(rect, ctx);
        }
    }

    pub fn on_click(&self, x: f32, y: f32, ctx: &RenderContext) -> Action {
        for element in self.elements.iter().rev() {
            let action = element.on_click(x, y, ctx);
            if action != Action::None {
                return action;
            }
        }
        Action::None
    }
}

pub struct SurfaceModel {
    pub role: SurfaceRole,
    pub tree: UiTree,
}

impl SurfaceModel {
    pub fn new(role: SurfaceRole, elements: Vec<Box<dyn Element>>) -> Self {
        Self {
            role,
            tree: UiTree::new(elements),
        }
    }
}

pub struct Backdrop {
    pub color: Color,
    pub width_factor: f32,
    pub height_factor: f32,
    pub inset_x: f32,
    pub inset_y: f32,
    pub radius: f32,
}

impl Backdrop {
    pub fn new(color: Color) -> Self {
        Self {
            color,
            width_factor: 1.0,
            height_factor: 1.0,
            inset_x: 0.0,
            inset_y: 0.0,
            radius: 18.0,
        }
    }
}

impl Element for Backdrop {
    fn draw(&self, rect: &RectProgram, ctx: &RenderContext) {
        let width = (ctx.surface_w * self.width_factor - self.inset_x * 2.0).max(0.0);
        let height = (ctx.surface_h * self.height_factor - self.inset_y * 2.0).max(0.0);
        let style = RectStyle {
            fill: self.color,
            fill_mode: FillMode::Solid,
            radius: Corners {
                tl: self.radius,
                tr: self.radius,
                br: self.radius,
                bl: self.radius,
            },
            softness: 1.0,
            ..Default::default()
        };

        rect.draw(
            ctx.surface_w,
            ctx.surface_h,
            width,
            height,
            &style,
            Mat3::translation(self.inset_x, self.inset_y),
        );
    }
}
