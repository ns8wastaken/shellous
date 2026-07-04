use crate::renderer::programs::rect::{Mat3, RectProgram, RectStyle};

// ==================== DRAWING SURFACE ====================

/// Drawing interface implemented by both `Canvas` (the rendering root) and
/// `TranslatedCanvas` (a translation-wrapped Canvas used by layout helpers).
///
/// `Element::draw` accepts `&dyn DrawingSurface`, so a Row can wrap each of
/// its children in a `TranslatedCanvas` and pass that in place of the raw
/// `Canvas` — child draw code stays unchanged.
pub trait DrawingSurface {
    fn draw_rect(
        &self,
        surface_w: f32,
        surface_h: f32,
        w: f32,
        h: f32,
        style: &RectStyle,
        transform: Mat3,
    );
}

// ==================== CANVAS ====================

/// Root drawing surface backed by the shared `RectProgram`. Constructed per
/// render frame by the Shell's draw loop.
pub struct Canvas<'a> {
    rect: &'a RectProgram,
}

impl<'a> Canvas<'a> {
    pub fn new(rect: &'a RectProgram) -> Self {
        Self { rect }
    }
    // `draw_rect` is intentionally not duplicated here — `DrawingSurface for
    // Canvas<'a>` is the only draw path, and every caller goes via
    // `&dyn DrawingSurface` (Row, layout helpers, and individual elements).
}

// Silence the unused-import lint for RectStyle (used only inside the trait
// impl, which other modules keep reachable through `&dyn DrawingSurface`).
#[allow(unused_imports)]
use RectStyle as _RectStyleForTraitDispatch;

impl<'a> DrawingSurface for Canvas<'a> {
    fn draw_rect(
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

// ==================== TRANSLATED CANVAS ====================

/// `DrawingSurface` wrapper that translates every `draw_rect` by `(dx, dy)`.
///
/// Used by `Row` (and future layout helpers) to position a child at a
/// computed offset without the child having to know its absolute position
/// on the surface. Lives here next to `Canvas`; conceptually a generic
/// layout helper primitive.
pub struct TranslatedCanvas<'a> {
    inner: &'a dyn DrawingSurface,
    dx: f32,
    dy: f32,
}

impl<'a> TranslatedCanvas<'a> {
    pub fn new(inner: &'a dyn DrawingSurface, dx: f32, dy: f32) -> Self {
        Self { inner, dx, dy }
    }
}

impl<'a> DrawingSurface for TranslatedCanvas<'a> {
    fn draw_rect(
        &self,
        surface_w: f32,
        surface_h: f32,
        w: f32,
        h: f32,
        style: &RectStyle,
        transform: Mat3,
    ) {
        // Compose: child transform applied first, then translation.
        // (Right-to-left matrix-vector: row_offset ∘ child_local.)
        let composed = Mat3::translation(self.dx, self.dy).multiply(&transform);
        self.inner
            .draw_rect(surface_w, surface_h, w, h, style, composed);
    }
}
