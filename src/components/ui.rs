use crate::components::canvas::DrawingSurface;
use crate::shell::state::ShellState;

// ==================== RENDER CONTEXT ====================

pub struct RenderContext<'a> {
    pub surface_w: f32,
    pub surface_h: f32,
    pub state: &'a ShellState,
    /// Total seconds elapsed since the shell started.
    pub absolute_time: f32,
}

// ==================== ELEMENT ====================

/// A drawable, clickable UI element on a shell surface.
pub trait Element {
    /// Tick animated properties. Return true if still animating.
    fn tick_animations(&mut self, _absolute_time: f32) -> bool {
        false
    }

    fn draw(&self, surface: &dyn DrawingSurface, ctx: &RenderContext);

    fn on_click(&self, x: f32, y: f32, ctx: &RenderContext) -> bool {
        let _ = (x, y, ctx);
        false
    }

    fn size(&self) -> (f32, f32) {
        (0.0, 0.0)
    }
}

// ==================== HELPERS ====================

pub fn tick_elements(elements: &mut [Box<dyn Element>], absolute_time: f32) -> bool {
    let mut active = false;
    for e in elements.iter_mut() {
        if e.tick_animations(absolute_time) {
            active = true;
        }
    }
    active
}

pub fn draw_elements(
    elements: &[Box<dyn Element>],
    surface: &dyn DrawingSurface,
    ctx: &RenderContext,
) {
    for element in elements {
        element.draw(surface, ctx);
    }
}

/// Iterates elements in reverse z-order (last-drawn = topmost for clicks).
/// Stops at the first element that returns `true` from `on_click`.
pub fn click_elements(elements: &[Box<dyn Element>], x: f32, y: f32, ctx: &RenderContext) {
    for element in elements.iter().rev() {
        if element.on_click(x, y, ctx) {
            return;
        }
    }
}
