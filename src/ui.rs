use crate::renderer::programs::rect::RectProgram;
use crate::shell::state::ShellState;
use crate::shell::surface_id::SurfaceId;

// ==================== ACTION ====================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    None,
    SwitchWorkspace(i32),
}

// ==================== ELEMENT ====================

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

// ==================== HELPERS ====================

pub fn draw_elements(elements: &[Box<dyn Element>], rect: &RectProgram, ctx: &RenderContext) {
    for element in elements {
        element.draw(rect, ctx);
    }
}

pub fn click_elements(elements: &[Box<dyn Element>], x: f32, y: f32, ctx: &RenderContext) -> Action {
    for element in elements.iter().rev() {
        let action = element.on_click(x, y, ctx);
        if action != Action::None {
            return action;
        }
    }
    Action::None
}
