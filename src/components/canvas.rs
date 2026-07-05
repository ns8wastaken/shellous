use crate::renderer::programs::rect::{Mat3, RectProgram, RectStyle};

// ==================== RECT ====================

#[derive(Clone, Copy, Debug, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn from_size(w: f32, h: f32) -> Self {
        Self { x: 0.0, y: 0.0, w, h }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.w && py >= self.y && py <= self.y + self.h
    }
}

// ==================== DRAWING SURFACE ====================

pub trait DrawingSurface {
    fn draw_rect(
        &self,
        w: f32,
        h: f32,
        style: &RectStyle,
        transform: Mat3,
    );
}

// ==================== CANVAS ====================

pub struct Canvas<'a> {
    rect: &'a RectProgram,
    surface_w: f32,
    surface_h: f32,
}

impl<'a> Canvas<'a> {
    pub fn new(rect: &'a RectProgram, surface_w: f32, surface_h: f32) -> Self {
        Self { rect, surface_w, surface_h }
    }
}

#[allow(unused_imports)]
use RectStyle as _RectStyleForTraitDispatch;

impl<'a> DrawingSurface for Canvas<'a> {
    fn draw_rect(
        &self,
        w: f32,
        h: f32,
        style: &RectStyle,
        transform: Mat3,
    ) {
        self.rect.draw(self.surface_w, self.surface_h, w, h, style, transform);
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
        w: f32,
        h: f32,
        style: &RectStyle,
        transform: Mat3,
    ) {
        let composed = Mat3::translation(self.dx, self.dy).multiply(&transform);
        self.inner
            .draw_rect(w, h, style, composed);
    }
}
