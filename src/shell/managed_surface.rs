use wayland_client::protocol::wl_surface::WlSurface;

use crate::renderer::Renderer;
use crate::shell::layer_surface::LayerSurface;
use crate::shell::state::ShellState;
use crate::shell::surface_id::SurfaceId;
use crate::ui::{RenderContext, SurfaceModel};

pub struct ManagedSurface {
    pub id: SurfaceId,
    pub layer: LayerSurface,
    pub wl_surface: WlSurface,
    pub renderer: Option<Renderer>,
    pub model: SurfaceModel,
}

impl ManagedSurface {
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
