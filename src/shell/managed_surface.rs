use std::cell::Cell;

use wayland_client::QueueHandle;

use crate::components::canvas::Canvas;
use crate::components::ui::{Element, RenderContext};
use crate::renderer::Renderer;
use crate::shell::state::ShellState;
use crate::shell::surface::{Surface, SurfaceKind};
use crate::shell::surface_id::SurfaceId;

pub struct ManagedSurface {
    pub id: SurfaceId,
    pub root: Option<Box<dyn Element>>,
    pub kind: SurfaceKind,
    pub renderer: Option<Renderer>,
    pub frame_pending: Cell<bool>,
    pub dirty: Cell<bool>,
}

impl ManagedSurface {
    pub fn render_context<'a>(
        &self,
        state: &'a ShellState,
        absolute_time: f32,
    ) -> RenderContext<'a> {
        let (w, h) = self.kind.dimensions();
        RenderContext {
            state,
            surface_w: w as f32,
            surface_h: h as f32,
            absolute_time,
        }
    }

    pub fn tick_animations(&mut self, absolute_time: f32) -> bool {
        self.root
            .as_mut()
            .map_or(false, |r| r.tick_animations(absolute_time))
    }

    pub fn request_frame(&self, qh: &QueueHandle<ShellState>) {
        if !self.frame_pending.get() {
            self.kind.wl_surface().frame(qh, self.id);
            self.frame_pending.set(true);
        }
    }

    pub fn draw(&self, canvas: &Canvas, ctx: &RenderContext) {
        if let Some(ref root) = self.root {
            root.draw(canvas, ctx);
        }
    }

    pub fn on_click(&self, x: f32, y: f32, ctx: &RenderContext) {
        if let Some(ref root) = self.root {
            root.on_click(x, y, ctx);
        }
    }
}
