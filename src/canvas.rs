use crate::renderer::programs::rect::{Mat3, RectProgram, RectStyle};

// ==================== CANVAS ====================

/// Drawing surface passed to UI elements.
///
/// Wraps the shader programs so elements don't depend on specific renderer
/// internals.  Add new drawing methods here when adding shaders (circle, text, …).
pub struct Canvas<'a> {
    rect: &'a RectProgram,
}

impl<'a> Canvas<'a> {
    pub fn new(rect: &'a RectProgram) -> Self {
        Self { rect }
    }

    pub fn draw_rect(
        &self,
        surface_w: f32,
        surface_h: f32,
        w: f32,
        h: f32,
        style: &RectStyle,
        transform: Mat3,
    ) {
        self.rect.draw(surface_w, surface_h, w, h, style, transform);
    }
}
