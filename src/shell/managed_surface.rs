use std::cell::Cell;

use wayland_client::QueueHandle;

use crate::components::arena::Slot;
use crate::components::layout_tree::LayoutNode;
use crate::components::ui::{Controller, RenderContext, ElementArena};
use crate::renderer::Renderer;
use crate::renderer::animation::cache::AnimationCache;
use crate::shell::state::ShellState;
use crate::shell::surface::{Surface, SurfaceKind};
use crate::shell::surface_id::SurfaceId;

pub struct ManagedSurface {
    pub id: SurfaceId,
    pub root: Option<Slot>,
    pub arena: ElementArena,
    pub kind: SurfaceKind,
    pub renderer: Option<Renderer>,
    pub frame_pending: Cell<bool>,
    pub dirty: Cell<bool>,
    pub animating: Cell<bool>,
    pub layout: Option<LayoutNode>,
    pub animations: AnimationCache,
    pub layout_dirty: Cell<bool>,
    pub controllers: Vec<Box<dyn Controller>>,
}

impl ManagedSurface {
    pub fn render_context<'a>(&'a self, state: &'a ShellState) -> RenderContext<'a> {
        let (w, h) = self.kind.dimensions();
        RenderContext {
            state,
            surface_w: w as f32,
            surface_h: h as f32,
            animations: &self.animations,
            arena: &self.arena,
        }
    }

    pub fn request_frame(&self, qh: &QueueHandle<ShellState>) {
        if !self.frame_pending.get() {
            self.kind.wl_surface().frame(qh, self.id);
            self.frame_pending.set(true);
        }
    }

    pub fn on_click(&self, x: f32, y: f32, ctx: &RenderContext) {
        for controller in &self.controllers {
            if let Some(layout) = &self.layout {
                if controller.on_click(x, y, &self.arena, layout, ctx) {
                    return;
                }
            }
        }
        if let (Some(root_slot), Some(layout)) = (self.root, &self.layout) {
            self.arena
                .get(root_slot)
                .unwrap()
                .on_click(layout, x, y, ctx);
        }
    }
}
