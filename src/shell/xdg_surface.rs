use std::sync::{Arc, Mutex};

use wayland_client::protocol::wl_surface::WlSurface;
use wayland_protocols::xdg::shell::client::{xdg_surface::XdgSurface, xdg_toplevel::XdgToplevel};

use crate::shell::surface::{Surface, SurfaceState};
use crate::shell::surface_id::SurfaceId;
use crate::shell::wayland::WaylandState;

/// xdg-shell toplevel surface — backed by `XdgWmBase`'s
/// `get_xdg_surface` + `get_toplevel`. Implements [`Surface`] so the same
/// render loop drives it as a layer-shell panel. Pass 2 scaffolding;
/// further protocol-level work (transient_for, decoration, close-driven
/// unmount) lives in later passes.
pub struct XdgToplevelSurface {
    pub wl_surface: WlSurface,
    /// xdg_surface proxy — kept for future surface-level operations
    /// (set_window_geometry, destroy). Not currently read.
    #[allow(dead_code)]
    pub xdg_surface: XdgSurface,
    pub xdg_toplevel: XdgToplevel,
    pub surface_state: Arc<Mutex<SurfaceState>>,
}

impl XdgToplevelSurface {
    pub fn new(
        wl: &WaylandState,
        surface_id: SurfaceId,
        title: &str,
        app_id: &str,
    ) -> Self {
        let surface_state = Arc::new(Mutex::new(SurfaceState {
            configured: false,
            width: 0,
            height: 0,
            surface_id,
        }));
        let qh = wl.qh();
        let wl_surface = wl.wl_compositor.create_surface(qh, ());
        let xdg_surface =
            wl.xdg_wm_base
                .get_xdg_surface(&wl_surface, qh, surface_state.clone());
        let xdg_toplevel = xdg_surface.get_toplevel(qh, surface_state.clone());
        xdg_toplevel.set_title(title.to_string());
        xdg_toplevel.set_app_id(app_id.to_string());
        Self {
            wl_surface,
            xdg_surface,
            xdg_toplevel,
            surface_state,
        }
    }
}

impl Surface for XdgToplevelSurface {
    fn dimensions(&self) -> (i32, i32) {
        let ss = self.surface_state.lock().unwrap();
        (ss.width, ss.height)
    }
    fn wl_surface(&self) -> &WlSurface {
        &self.wl_surface
    }
    fn surface_state(&self) -> &Arc<Mutex<SurfaceState>> {
        &self.surface_state
    }
}
