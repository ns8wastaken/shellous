use std::sync::{Arc, Mutex};

use crate::components::bar::BarState;
use crate::shell::compositor::Compositor;
use crate::shell::managed_surface::ManagedSurface;
use crate::shell::surface_id::SurfaceId;

pub struct ShellState {
    pub bar: Arc<Mutex<BarState>>,
    pub compositor: Arc<dyn Compositor>,
    pub surfaces: Vec<ManagedSurface>,
    pub focused_surface: Option<usize>,
    pub pointer_pos: Option<(f64, f64)>,
    pub next_id: SurfaceId,
}

impl ShellState {
    pub fn new(compositor: Arc<dyn Compositor>) -> Self {
        Self {
            bar: Arc::new(Mutex::new(BarState {
                workspaces: Vec::new(),
                active_id: -1,
            })),
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

    pub fn pointer_pos_for(&self, id: SurfaceId) -> Option<(f64, f64)> {
        match self.focused_surface {
            Some(focused_idx) if self.surfaces.get(focused_idx)?.id == id => self.pointer_pos,
            _ => None,
        }
    }

    pub fn handle_click(&self) {
        let idx = match self.focused_surface {
            Some(i) => i,
            None => return,
        };
        let (x, y) = match self.pointer_pos {
            Some((x, y)) => (x as f32, y as f32),
            None => return,
        };

        let surface = &self.surfaces[idx];
        let ctx = surface.render_context(self);
        surface.on_click(x, y, &ctx);
    }

    pub fn set_focus_by_surface(
        &mut self,
        wl_surface: &wayland_client::protocol::wl_surface::WlSurface,
    ) {
        self.focused_surface = self.surfaces.iter().position(|s| s.wl_surface == *wl_surface);
        eprintln!(
            "[shell] focus -> surface {:?}",
            self.focused_surface.map(|i| self.surfaces[i].id)
        );
    }


}
