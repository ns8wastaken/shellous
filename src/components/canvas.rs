use crate::renderer::programs::rect::{Mat3, RectProgram, RectStyle};

// ==================== DRAWING SURFACE ====================

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

pub struct Canvas<'a> {
    rect: &'a RectProgram,
}

impl<'a> Canvas<'a> {
    pub fn new(rect: &'a RectProgram) -> Self {
        Self { rect }
    }
}

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
        let composed = Mat3::translation(self.dx, self.dy).multiply(&transform);
        self.inner
            .draw_rect(surface_w, surface_h, w, h, style, composed);
    }
}
