use std::sync::{Arc, Mutex};

use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_compositor::WlCompositor, wl_seat::WlSeat, wl_surface::WlSurface},
    Connection, EventQueue, QueueHandle,
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{Anchor, ZwlrLayerSurfaceV1},
};

use crate::shell::state::ShellState;
use crate::shell::surface_id::SurfaceId;

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

// ==================== SURFACE STATE ====================

pub struct SurfaceState {
    pub configured: bool,
    pub width: i32,
    pub height: i32,
    #[allow(dead_code)]
    pub surface_id: SurfaceId,
}

pub struct WaylandState {
    pub conn: Connection,
    event_queue: EventQueue<ShellState>,
    qh: QueueHandle<ShellState>,
    pub layer_shell: ZwlrLayerShellV1,
    wl_compositor: WlCompositor,
    #[allow(dead_code)]
    seat: WlSeat,
}

impl WaylandState {
    pub fn new() -> Self {
        let conn = Connection::connect_to_env().unwrap();
        let (globals, event_queue) = registry_queue_init::<ShellState>(&conn).unwrap();
        let qh = event_queue.handle();

        let wl_compositor = globals
            .bind::<WlCompositor, _, _>(&qh, 1..=5, ())
            .expect("wl_compositor not available");
        let layer_shell = globals
            .bind::<ZwlrLayerShellV1, _, _>(&qh, 1..=4, ())
            .expect("zwlr_layer_shell_v1 not available");
        let seat = globals
            .bind::<WlSeat, _, _>(&qh, 1..=8, ())
            .expect("wl_seat not available");

        Self {
            conn,
            event_queue,
            qh,
            layer_shell,
            wl_compositor,
            seat,
        }
    }

    /// Non-blocking: dispatch any events already buffered.
    pub fn dispatch_pending(&mut self, state: &mut ShellState) {
        self.event_queue.dispatch_pending(state).unwrap();
    }

    /// Block until the next Wayland event arrives, then dispatch it.
    pub fn blocking_dispatch(&mut self, state: &mut ShellState) {
        self.event_queue.blocking_dispatch(state).unwrap();
    }

    pub fn qh(&self) -> &QueueHandle<ShellState> {
        &self.qh
    }

    pub fn wait_for_configure(
        &mut self,
        state: &mut ShellState,
        surface_state: &Arc<Mutex<SurfaceState>>,
    ) {
        loop {
            if surface_state.lock().unwrap().configured {
                break;
            }
            self.event_queue.blocking_dispatch(state).unwrap();
        }
    }
}

pub struct LayerSurface {
    pub layer_surface: ZwlrLayerSurfaceV1,
    pub surface_state: Arc<Mutex<SurfaceState>>,
}

impl LayerSurface {
    pub fn dimensions(&self) -> (i32, i32) {
        let ss = self.surface_state.lock().unwrap();
        (ss.width, ss.height)
    }

    pub fn new(
        wl: &WaylandState,
        namespace: &str,
        surface_id: SurfaceId,
        layer: Layer,
    ) -> (Self, WlSurface) {
        let qh = &wl.qh;
        let surface_state = Arc::new(Mutex::new(SurfaceState {
            configured: false,
            width: 0,
            height: 0,
            surface_id,
        }));

        let surface = wl.wl_compositor.create_surface(qh, ());
        let layer_surface = wl.layer_shell.get_layer_surface(
            &surface,
            None,
            layer,
            namespace.to_string(),
            qh,
            surface_state.clone(),
        );

        (
            Self {
                layer_surface,
                surface_state,
            },
            surface,
        )
    }
}
