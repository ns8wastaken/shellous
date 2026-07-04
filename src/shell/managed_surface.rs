use std::cell::Cell;

use wayland_client::QueueHandle;

use crate::canvas::Canvas;
use crate::renderer::Renderer;
use crate::shell::state::ShellState;
use crate::shell::surface::{Surface, SurfaceKind};
use crate::shell::surface_id::SurfaceId;
use crate::ui::{Element, RenderContext, click_elements, draw_elements};

pub struct ManagedSurface {
    pub id: SurfaceId,
    pub elements: Vec<Box<dyn Element>>,
    pub kind: SurfaceKind,
    pub renderer: Option<Renderer>,
    pub frame_pending: Cell<bool>,
    pub dirty: Cell<bool>,
}

impl ManagedSurface {
    pub fn render_context<'a>(&self, state: &'a ShellState) -> RenderContext<'a> {
        let (w, h) = self.kind.dimensions();
        RenderContext {
            state,
            surface_id: self.id,
            surface_w: w as f32,
            surface_h: h as f32,
            pointer_pos: state.pointer_pos_for(self.id),
        }
    }

    pub fn request_frame(&self, qh: &QueueHandle<ShellState>) {
        if !self.frame_pending.get() {
            self.kind.wl_surface().frame(qh, self.id);
            self.frame_pending.set(true);
        }
    }

    pub fn draw(&self, canvas: &Canvas, ctx: &RenderContext) {
        draw_elements(&self.elements, canvas, ctx);
    }

    pub fn on_click(&self, x: f32, y: f32, ctx: &RenderContext) {
        click_elements(&self.elements, x, y, ctx);
    }
}
