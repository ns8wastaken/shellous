use std::sync::{Arc, Mutex};

use wayland_client::protocol::wl_surface::WlSurface;

use crate::shell::layer_surface::LayerSurface;
use crate::shell::surface_id::SurfaceId;
use crate::shell::xdg_surface::XdgToplevelSurface;

// ==================== SURFACE STATE ====================

/// Per-surface state populated by Wayland `Configure` events and read by
/// the renderer and dispatch machinery. Shared between the protocol
/// dispatch impls (writer) and the render loop (reader) via
/// `Arc<Mutex<…>>`.
pub struct SurfaceState {
    pub configured: bool,
    pub width: i32,
    pub height: i32,
    pub surface_id: SurfaceId,
}

// ==================== SURFACE TRAIT ====================

/// A mounted Wayland surface. Provides the data the render loop needs to
/// run a frame on this surface — its reported dimensions, the raw
/// `WlSurface` proxy for commit/lifecycle, and the shared state handle
/// for protocol dispatch (configure, ack, etc).
pub trait Surface: Send {
    fn dimensions(&self) -> (i32, i32);
    fn wl_surface(&self) -> &WlSurface;
    fn surface_state(&self) -> &Arc<Mutex<SurfaceState>>;
}

// ==================== SURFACE KIND ====================

/// Polymorphic surface. `Layer` covers zwlr-layer-shell panels (used by
/// the bar). `Toplevel` is xdg-shell scaffolding for future toplevel
/// windows. New variants (`Popup`, `Subsurface`) will slot in here as
/// their scaffolding lands.
pub enum SurfaceKind {
    Layer(LayerSurface),
    Toplevel(XdgToplevelSurface),
}

impl Surface for SurfaceKind {
    fn dimensions(&self) -> (i32, i32) {
        match self {
            Self::Layer(l) => l.dimensions(),
            Self::Toplevel(t) => t.dimensions(),
        }
    }
    fn wl_surface(&self) -> &WlSurface {
        match self {
            Self::Layer(l) => l.wl_surface(),
            Self::Toplevel(t) => t.wl_surface(),
        }
    }
    fn surface_state(&self) -> &Arc<Mutex<SurfaceState>> {
        match self {
            Self::Layer(l) => l.surface_state(),
            Self::Toplevel(t) => t.surface_state(),
        }
    }
}
