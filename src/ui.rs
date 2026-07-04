use crate::canvas::DrawingSurface;
use crate::shell::state::ShellState;
use crate::shell::surface_id::SurfaceId;

// ==================== ELEMENT ====================

pub struct RenderContext<'a> {
    pub state: &'a ShellState,
    pub surface_id: SurfaceId,
    pub surface_w: f32,
    pub surface_h: f32,
    pub pointer_pos: Option<(f64, f64)>,
}

/// A drawable, clickable UI element on a shell surface.
///
/// `on_click` receives pixel coordinates relative to the surface origin.
/// Return `true` if the click was handled (stops iteration over elements);
/// return `false` to let the next element in z-order try.
///
/// `size` returns the element's bounding box in its own local coordinate
/// system. Override this if you want the element to participate in a `Row`
/// (or other layout helper) — the default `(0, 0)` makes the element take
/// no space in any parent layout.
pub trait Element {
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

pub fn draw_elements(elements: &[Box<dyn Element>], surface: &dyn DrawingSurface, ctx: &RenderContext) {
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
