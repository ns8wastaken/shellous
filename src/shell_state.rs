use std::sync::{Arc, Mutex};

use crate::action::Action;
use crate::bar::BarState;
use crate::compositor::Compositor;
use crate::managed_surface::ManagedSurface;
use crate::surface_id::SurfaceId;

// ==================== SHELL STATE ====================

/// Global shell state — the Dispatch type for the Wayland event queue.
///
/// Owns all managed surfaces, shared compositor data, and pointer focus.
/// Events are routed through the Wayland `Dispatch` impls (in `wayland.rs`)
/// which call methods on this struct.
pub struct ShellState {
    /// Workspace list + active workspace (filled by the compositor backend).
    pub bar: Arc<Mutex<BarState>>,
    /// Back-end for workspace switching (Hyprland currently).
    pub compositor: Arc<dyn Compositor>,
    /// All registered layer surfaces.
    pub surfaces: Vec<ManagedSurface>,
    /// Index into `surfaces` of the surface currently under the pointer.
    pub focused_surface: Option<usize>,
    /// Pointer coordinates relative to the focused surface.
    pub pointer_pos: Option<(f64, f64)>,
    /// Monotonically increasing counter for assigning `SurfaceId`s.
    pub next_id: SurfaceId,
}

impl ShellState {
    /// Create a fresh shell state (no surfaces yet).
    pub fn new(
        bar: Arc<Mutex<BarState>>,
        compositor: Arc<dyn Compositor>,
    ) -> Self {
        Self {
            bar,
            compositor,
            surfaces: Vec::new(),
            focused_surface: None,
            pointer_pos: None,
            next_id: 0,
        }
    }

    /// Register a new surface and return its id.
    pub fn register(&mut self, surface: ManagedSurface) -> SurfaceId {
        let id = surface.id;
        self.surfaces.push(surface);
        id
    }

    /// Look up a surface by id.
    pub fn find_surface(&self, id: SurfaceId) -> Option<&ManagedSurface> {
        self.surfaces.iter().find(|s| s.id == id)
    }

    /// Look up a surface mutably by id.
    pub fn find_surface_mut(&mut self, id: SurfaceId) -> Option<&mut ManagedSurface> {
        self.surfaces.iter_mut().find(|s| s.id == id)
    }

    /// Return the pointer position if `id` is the currently focused surface.
    pub fn pointer_pos_for(&self, id: SurfaceId) -> Option<(f64, f64)> {
        match self.focused_surface {
            Some(focused_idx) => {
                let focused = self.surfaces.get(focused_idx)?;
                if focused.id == id {
                    self.pointer_pos
                } else {
                    None
                }
            }
            None => None,
        }
    }

    // ==================== CLICK HANDLING ====================

    /// Handle a pointer button press on the currently focused surface.
    /// Delegates to each panel's `on_click` method until one returns
    /// a non-`None` action, then executes that action.
    pub fn handle_click(&self) {
        let idx = match self.focused_surface {
            Some(i) => i,
            None => return,
        };
        let (x, y) = match self.pointer_pos {
            Some(p) => (p.0 as f32, p.1 as f32),
            None => return,
        };

        let surface = &self.surfaces[idx];
        let ctx = surface.render_context(self);

        for panel in &surface.panels {
            match panel.on_click(x, y, &ctx) {
                Action::SwitchWorkspace(id) => {
                    eprintln!("[shell] switching to workspace {id}");
                    self.compositor.switch_workspace(id);
                    break;
                }
                Action::None => continue,
            }
        }
    }

    // ==================== POINTER FOCUS ====================

    /// Set the focused surface by matching a Wayland `WlSurface` object.
    /// Called from the `WlPointer::Enter` dispatch handler.
    pub fn set_focus_by_surface(
        &mut self,
        wl_surface: &wayland_client::protocol::wl_surface::WlSurface,
    ) {
        self.focused_surface = self
            .surfaces
            .iter()
            .position(|s| s.wl_surface == *wl_surface);
        eprintln!(
            "[shell] focus -> surface {:?}",
            self.focused_surface.map(|i| self.surfaces[i].id)
        );
    }
}
