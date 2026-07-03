use wayland_client::QueueHandle;
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
    pub frame_pending: bool,   // true while waiting on a frame callback
    pub dirty: bool,           // true if something changed and we want to redraw ASAP
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

    pub fn request_frame(&mut self, qh: &QueueHandle<ShellState>) {
        if !self.frame_pending {
            self.wl_surface.frame(qh, self.id); // id as user-data so Dispatch<WlCallback> knows which surface
            self.frame_pending = true;
        }
    }
}
