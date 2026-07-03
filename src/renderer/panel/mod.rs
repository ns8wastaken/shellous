pub mod bar;

use crate::action::Action;
use crate::shell_state::ShellState;
use crate::surface_id::SurfaceId;
use crate::renderer::programs::rect::RectProgram;

// ==================== RENDER CONTEXT ====================

/// Context passed to every Panel::draw / on_click call.
/// Bundles shared state plus per-surface metadata so the trait signature
/// stays stable as the shell grows.
pub struct RenderContext<'a> {
    /// Global shell state (workspaces, compositor, etc.).
    pub state: &'a ShellState,
    /// Which surface this panel is rendering for.
    #[allow(dead_code)]
    pub surface_id: SurfaceId,
    /// Surface dimensions in logical pixels.
    pub surface_w: f32,
    pub surface_h: f32,
    /// Pointer position, if this surface currently has pointer focus.
    pub pointer_pos: Option<(f64, f64)>,
}

// ==================== PANEL TRAIT ====================

/// A single panel drawn on a layer surface.
/// Each panel has a position, size, background, and content drawn inside it.
pub trait Panel {
    /// Draw this panel into the current GL context.
    fn draw(&self, rect: &RectProgram, ctx: &RenderContext);

    /// Handle a click at (`x`, `y`) in surface-local coordinates.
    /// Return an `Action` to tell the shell what to do, or `Action::None`
    /// to let the next panel try.
    fn on_click(&self, x: f32, y: f32, ctx: &RenderContext) -> Action {
        let _ = (x, y, ctx);
        Action::None
    }
}
