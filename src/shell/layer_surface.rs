use std::sync::{Arc, Mutex};

use wayland_client::protocol::wl_surface::WlSurface;
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::Layer,
    zwlr_layer_surface_v1::{Anchor, ZwlrLayerSurfaceV1},
};

use crate::shell::surface::{Surface, SurfaceState};
use crate::shell::surface_id::SurfaceId;
use crate::shell::wayland::WaylandState;

// ==================== SHELL-LEVEL WRAPPER TYPES ====================

/// Shell-level layer ordering, mirrors `zwlr_layer_shell_v1::Layer`.
/// Components use this instead of importing wayland protocol types directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellLayer {
    Background,
    Bottom,
    Top,
    Overlay,
}

impl ShellLayer {
    pub(crate) fn to_wayland(self) -> Layer {
        match self {
            ShellLayer::Background => Layer::Background,
            ShellLayer::Bottom     => Layer::Bottom,
            ShellLayer::Top        => Layer::Top,
            ShellLayer::Overlay    => Layer::Overlay,
        }
    }
}

/// Shell-level anchor bit-flag, mirrors `zwlr_layer_surface_v1::Anchor`.
#[derive(Debug, Clone, Copy)]
pub struct ShellAnchor(u32);

impl ShellAnchor {
    pub const TOP: ShellAnchor    = ShellAnchor(1);
    pub const BOTTOM: ShellAnchor = ShellAnchor(2);
    pub const LEFT: ShellAnchor   = ShellAnchor(4);
    pub const RIGHT: ShellAnchor  = ShellAnchor(8);

    pub(crate) fn to_wayland(self) -> Anchor {
        Anchor::from_bits_truncate(self.0)
    }
}

impl std::ops::BitOr for ShellAnchor {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        ShellAnchor(self.0 | rhs.0)
    }
}

// ==================== LAYER SURFACE ====================

/// Layer-shell Wayland object plus its primitive-state and the underlying
/// wl_surface. Implements [`Surface`] for the layer-shell protocol.
pub struct LayerSurface {
    pub layer_surface: ZwlrLayerSurfaceV1,
    pub wl_surface: WlSurface,
    pub surface_state: Arc<Mutex<SurfaceState>>,
}

impl LayerSurface {
    pub fn new(
        wl: &WaylandState,
        surface_id: SurfaceId,
        namespace: &str,
        layer: Layer,
        anchor: Anchor,
        width: u32,
        height: u32,
        exclusive_zone: i32,
    ) -> Self {
        let surface_state = Arc::new(Mutex::new(SurfaceState {
            configured: false,
            width: 0,
            height: 0,
            surface_id,
        }));
        let qh = wl.qh();
        let wl_surface = wl.wl_compositor.create_surface(qh, ());
        let layer_surface = wl.layer_shell.get_layer_surface(
            &wl_surface,
            None,
            layer,
            namespace.to_string(),
            qh,
            surface_state.clone(),
        );
        layer_surface.set_anchor(anchor);
        layer_surface.set_size(width, height);
        layer_surface.set_exclusive_zone(exclusive_zone);
        Self {
            layer_surface,
            wl_surface,
            surface_state,
        }
    }
}

impl Surface for LayerSurface {
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
