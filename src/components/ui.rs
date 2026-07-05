use crate::components::canvas::DrawingSurface;
use crate::services::workspace::WorkspaceSnapshot;
use crate::shell::state::ShellState;

// ==================== RENDER CONTEXT ====================

pub struct RenderContext<'a> {
    pub surface_w: f32,
    pub surface_h: f32,
    pub state: &'a ShellState,
}

// ==================== ELEMENT ====================

/// A drawable, clickable UI element on a shell surface.
pub trait Element {
    /// Push new data into the element. Called before tick_animations.
    fn update(&mut self, _snapshot: &WorkspaceSnapshot) {}

    /// Tick animated properties. Return true if still animating.
    fn tick_animations(&mut self, _absolute_time: f32) -> bool {
        false
    }

    fn draw(&self, surface: &dyn DrawingSurface, ctx: &RenderContext);

    fn on_click(&self, x: f32, y: f32, ctx: &RenderContext) -> bool {
        let _ = (x, y, ctx);
        false
    }

    fn id(&self) -> Option<i32> {
        None
    }

    fn size(&self) -> (f32, f32) {
        (0.0, 0.0)
    }
}
