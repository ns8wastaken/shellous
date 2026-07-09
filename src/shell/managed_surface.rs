use std::cell::Cell;

use wayland_client::QueueHandle;

use crate::components::layout_tree::LayoutNode;
use crate::components::rect::Size;
use crate::components::ui::{Element, RenderContext};
use crate::shell::state::ShellState;
use crate::shell::surface::{Surface, SurfaceKind};
use crate::shell::surface_id::SurfaceId;

pub struct ManagedSurface {
    pub id: SurfaceId,
    pub root: Option<Box<dyn Element>>,
    pub kind: SurfaceKind,
    pub renderer: Option<crate::renderer::Renderer>,
    pub frame_pending: Cell<bool>,
    pub dirty: Cell<bool>,
    pub layout: Option<LayoutNode>,
}

impl ManagedSurface {
    pub fn render_context<'a>(&self, state: &'a ShellState) -> RenderContext<'a> {
        let (w, h) = self.kind.dimensions();
        RenderContext {
            state,
            surface_w: w as f32,
            surface_h: h as f32,
        }
    }

    pub fn root_size(&self) -> Size {
        let (w, h) = self.kind.dimensions();
        Size { w: w as f32, h: h as f32 }
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

    pub fn on_click(&self, x: f32, y: f32, ctx: &RenderContext) {
        if let (Some(root), Some(layout)) = (&self.root, &self.layout) {
            root.on_click(layout, x, y, ctx);
        }
    }
}
