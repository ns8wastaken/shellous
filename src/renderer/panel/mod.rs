pub mod bar;

use crate::display::AppState;
use crate::renderer::programs::rect::RectProgram;

/// A single panel drawn on the bar surface.
/// Each panel has a position, size, background, and content drawn inside it.
pub trait Panel {
    /// Draw this panel into the current GL context.
    fn draw(&self, rect: &RectProgram, surface_w: f32, surface_h: f32, state: &AppState);
}
