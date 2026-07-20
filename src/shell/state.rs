use std::sync::Arc;

use crate::components::geom::{Rect, Size};

use crate::renderer::batch::DrawBatch;
use crate::renderer::animation::cache::AnimationCache;
use crate::shell::compositor::Compositor;
use crate::shell::event::ShellEvent;
use crate::shell::managed_surface::ManagedSurface;
use crate::shell::surface::Surface;
use crate::shell::surface_id::SurfaceId;

pub struct ShellState {
    pub compositor: Arc<dyn Compositor>,
    pub surfaces: Vec<ManagedSurface>,
    pub focused_surface: Option<SurfaceId>,
    pub pointer_pos: Option<(f64, f64)>,
    pub next_id: SurfaceId,
}

impl ShellState {
    pub fn new(compositor: Arc<dyn Compositor>) -> Self {
        Self {
            compositor,
            surfaces: Vec::new(),
            focused_surface: None,
            pointer_pos: None,
            next_id: 0,
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

    pub fn any_dirty(&self) -> bool {
        self.surfaces.iter().any(|s| s.dirty.get())
    }

    pub fn update_surfaces(&mut self, event: &ShellEvent, now: f32) {
        for entry in &mut self.surfaces {
            let ManagedSurface {
                controllers,
                arena,
                animations,
                dirty,
                layout_dirty,
                ..
            } = entry;

            let mut any_changed = false;
            for ctrl in controllers.iter_mut() {
                if ctrl.update(event, now, animations, arena) {
                    any_changed = true;
                }
            }

            if any_changed {
                let (w, h) = entry.kind.dimensions();
                for ctrl in controllers.iter() {
                    ctrl.sync(now, animations, arena, w as f32, h as f32);
                }
                dirty.set(true);
                layout_dirty.set(true);
            }
        }
    }

    pub fn tick_animations(&mut self, now: f32) -> bool {
        let mut still_moving = false;
        for entry in &mut self.surfaces {
            if entry.renderer.is_none() {
                continue;
            }
            let ManagedSurface {
                controllers,
                arena,
                animations,
                animating,
                dirty,
                layout_dirty,
                ..
            } = entry;

            let active = animations.tick(now);
            if active {
                let (w, h) = entry.kind.dimensions();
                for ctrl in controllers.iter() {
                    ctrl.sync(now, animations, arena, w as f32, h as f32);
                }
                animating.set(true);
                dirty.set(true);
                layout_dirty.set(true);
                still_moving = true;
            } else {
                animating.set(false);
            }
        }
        still_moving
    }

    pub fn compute_layouts(&mut self) {
        for entry in &mut self.surfaces {
            if !entry.layout_dirty.get() {
                continue;
            }
            let ManagedSurface {
                root,
                arena,
                layout,
                kind,
                animations,
                layout_dirty,
                ..
            } = entry;
            let (w, h) = kind.dimensions();
            let root_size = Size { w: w as f32, h: h as f32 };
            if let Some(root_slot) = root {
                let cache: &AnimationCache = animations;
                // layout() is still called so nested Align/containers can
                // compute child sizes, but the root rect always spans the
                // full compositor-assigned surface dimensions.
                arena.get(*root_slot)
                    .unwrap()
                    .layout(root_size, cache, arena);
                let root_rect = Rect::from_size(root_size);
                *layout = Some(
                    arena
                        .get(*root_slot)
                        .unwrap()
                        .layout_tree(root_rect, cache, arena),
                );
            }
            layout_dirty.set(false);
        }
    }

    pub fn render(&self) {
        for entry in &self.surfaces {
            if !entry.dirty.get() || entry.renderer.is_none() {
                continue;
            }
            let renderer = entry.renderer.as_ref().unwrap();
            renderer.make_current();
            let ctx = entry.render_context(self);

            let mut batch = DrawBatch::new();
            if let (Some(root_slot), Some(layout)) = (entry.root, &entry.layout) {
                entry
                    .arena
                    .get(root_slot)
                    .unwrap()
                    .draw(layout, &mut batch, &ctx);
            }

            batch.sort_by_shape();

            renderer.render_frame(|| {
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
