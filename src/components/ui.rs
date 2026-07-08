use crate::components::canvas::{Rect, Size};
use crate::renderer::batch::DrawBatch;
use crate::services::workspace::WorkspaceSnapshot;
use crate::shell::state::ShellState;

// ==================== RENDER CONTEXT ====================

pub struct RenderContext<'a> {
    pub surface_w: f32,
    pub surface_h: f32,
    pub state: &'a ShellState,
}

// ==================== ELEMENT ====================

pub trait Element {
    fn update(&mut self, _snapshot: &WorkspaceSnapshot) {}

    fn tick_animations(&mut self, _absolute_time: f32) -> bool {
        false
    }

    /// Compute the element's desired size given available space.
    /// Called during the layout phase (CPU only).
    fn layout(&self, available: Size) -> Size;

    /// Collect draw commands into `batch` for the absolute `rect` region.
    /// Called during the geometry batching phase (CPU memory).
    fn draw(&self, rect: Rect, batch: &mut DrawBatch, ctx: &RenderContext);

    fn on_click(&self, rect: Rect, x: f32, y: f32, ctx: &RenderContext) -> bool {
        let _ = (rect, x, y, ctx);
        false
    }

    fn id(&self) -> Option<i32> {
        None
    }
}
