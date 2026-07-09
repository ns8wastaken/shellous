use std::cell::Cell;
use std::sync::Arc;

use crate::components::rect::Rect;
use crate::renderer::batch::DrawBatch;
use crate::services::workspace::WorkspaceSnapshot;
use crate::shell::compositor::Compositor;
use crate::shell::managed_surface::ManagedSurface;
use crate::shell::surface::Surface;
use crate::shell::surface_id::SurfaceId;

pub struct ShellState {
    pub compositor: Arc<dyn Compositor>,
    pub surfaces: Vec<ManagedSurface>,
    pub focused_surface: Option<SurfaceId>,
    pub pointer_pos: Option<(f64, f64)>,
    pub next_id: SurfaceId,
    pub animating: Cell<bool>,
}

impl ShellState {
    pub fn new(compositor: Arc<dyn Compositor>) -> Self {
        Self {
            compositor,
            surfaces: Vec::new(),
            focused_surface: None,
            pointer_pos: None,
            next_id: 0,
            animating: Cell::new(false),
        }
    }

    pub fn register(&mut self, surface: ManagedSurface) -> SurfaceId {
        let id = surface.id;
        self.surfaces.push(surface);
        id
    }

    pub fn find_surface(&self, id: SurfaceId) -> Option<&ManagedSurface> {
        self.surfaces.iter().find(|s| s.id == id)
    }

    pub fn find_surface_mut(&mut self, id: SurfaceId) -> Option<&mut ManagedSurface> {
        self.surfaces.iter_mut().find(|s| s.id == id)
    }

    pub fn pointer_pos_for(&self, id: SurfaceId) -> Option<(f64, f64)> {
        match self.focused_surface {
            Some(focused_id) if focused_id == id => self.pointer_pos,
            _ => None,
        }
    }

    pub fn is_animating(&self) -> bool {
        self.animating.get()
    }

    pub fn set_animating(&self, v: bool) {
        self.animating.set(v);
    }

    pub fn any_dirty(&self) -> bool {
        self.surfaces.iter().any(|s| s.dirty.get())
    }

    /// Called after eventfd wake. Kicks off the animation cycle.
    pub fn sync_workspace_snapshots(&mut self) {
        for entry in &mut self.surfaces {
            entry.dirty.set(true);
        }
        self.animating.set(true);
    }

    /// Push a fresh workspace snapshot through the element tree.
    pub fn update_surfaces(&mut self, snapshot: &WorkspaceSnapshot) {
        for entry in &mut self.surfaces {
            if let Some(ref mut root) = entry.root {
                root.update(snapshot);
            }
        }
    }

    /// Update phase — tick all element animations. Returns true if any
    /// animation is still active.
    pub fn tick_animations(&mut self, absolute_time: f32) -> bool {
        let mut still_moving = false;
        for entry in &mut self.surfaces {
            if entry.renderer.is_some() && entry.dirty.get() {
                if entry.tick_animations(absolute_time) {
                    still_moving = true;
                }
            }
        }
        still_moving
    }

    /// Build cached layout tree for all dirty surfaces.
    /// Called after tick_animations, before render.
    pub fn compute_layouts(&mut self) {
        for entry in &mut self.surfaces {
            if !entry.dirty.get() {
                continue;
            }
            let root_size = entry.root_size();
            if let Some(root) = entry.root.as_ref() {
                let desired = root.layout(root_size);
                let root_rect = Rect::from_size(desired);
                entry.layout = Some(root.layout_tree(root_rect));
            }
        }
    }

    /// Render phase — two-pass pipeline (layout is already cached):
    ///   1. Geometry batching (CPU memory)
    ///   2. GPU render
    pub fn render(&self) {
        for entry in &self.surfaces {
            if !entry.dirty.get() || entry.renderer.is_none() {
                continue;
            }
            let renderer = entry.renderer.as_ref().unwrap();
            renderer.make_current();
            let ctx = entry.render_context(self);

            // Pass 1: Geometry batching (CPU memory) from cached layout
            let mut batch = DrawBatch::new();
            if let (Some(root), Some(layout)) = (&entry.root, &entry.layout) {
                root.draw(layout, &mut batch, &ctx);
            }

            // Sort by shape so the GPU dispatch hits each program once
            batch.sort_by_shape();

            // Pass 2: GPU render
            renderer.render_frame(&ctx, || {
                renderer.render_batch(&batch, ctx.surface_w, ctx.surface_h);
            });
        }
    }

    pub fn handle_click(&self) {
        let id = match self.focused_surface {
            Some(id) => id,
            None => return,
        };
        let (x, y) = match self.pointer_pos {
            Some((x, y)) => (x as f32, y as f32),
            None => return,
        };

        let surface = match self.find_surface(id) {
            Some(s) => s,
            None => return,
        };
        let ctx = surface.render_context(self);
        surface.on_click(x, y, &ctx);
    }

    pub fn set_focus_by_surface(
        &mut self,
        wl_surface: &wayland_client::protocol::wl_surface::WlSurface,
    ) {
        self.focused_surface = self
            .surfaces
            .iter()
            .find(|s| s.kind.wl_surface() == wl_surface)
            .map(|s| s.id);
    }
}
