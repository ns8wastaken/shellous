use wayland_client::protocol::wl_surface::WlSurface;

use crate::layer_surface::LayerSurface;
use crate::renderer::panel::{Panel, RenderContext};
use crate::renderer::Renderer;
use crate::shell_state::ShellState;
use crate::surface_id::SurfaceId;

// ==================== MANAGED SURFACE ====================

/// Everything needed for one layer surface: Wayland protocol objects,
/// EGL renderer, and the panels drawn on it.
///
/// The `renderer` starts as `None` and is filled in after the compositor
/// sends the initial `Configure` event (which provides the real dimensions).
pub struct ManagedSurface {
    /// Unique id assigned by the Shell.
    pub id: SurfaceId,
    /// Wayland layer-surface protocol objects.
    pub layer: LayerSurface,
    /// The Wayland surface backing this layer.
    pub wl_surface: WlSurface,
    /// EGL renderer — `None` until after `wait_for_configure`.
    pub renderer: Option<Renderer>,
    /// Panels drawn on this surface (also handle clicks).
    pub panels: Vec<Box<dyn Panel>>,
}

impl ManagedSurface {
    /// Build a `RenderContext` suitable for passing to `Panel` methods.
    pub fn render_context<'a>(&self, state: &'a ShellState) -> RenderContext<'a> {
        let (w, h) = self.layer.dimensions();
        RenderContext {
            state,
            surface_id: self.id,
            surface_w: w as f32,
            surface_h: h as f32,
            pointer_pos: state.pointer_pos_for(self.id),
        }
    }
}
