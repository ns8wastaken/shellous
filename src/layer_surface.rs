use std::sync::{Arc, Mutex};

use wayland_client::{
    Connection, EventQueue, QueueHandle,
    globals::registry_queue_init,
    protocol::{
        wl_compositor::WlCompositor,
        wl_seat::WlSeat,
        wl_surface::WlSurface,
    },
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
};

use crate::shell_state::ShellState;
use crate::surface_id::SurfaceId;

// ==================== SURFACE STATE ====================

/// Per-surface state — each layer surface gets its own instance.
/// Configured by the compositor's Configure event (not by the client's request).
pub struct SurfaceState {
    pub configured: bool,
    pub width: i32,
    pub height: i32,
    /// Which managed surface this state belongs to (available for Dispatch routing).
    #[allow(dead_code)]
    pub surface_id: SurfaceId,
}

// ==================== SHARED WAYLAND STATE ====================

/// A single Wayland connection + event queue shared by all surfaces.
/// Owns the global bindings (compositor, layer shell, seat) that are needed
/// to create individual layer surfaces.
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
    /// Connect to Wayland and bind globals.
    /// All surfaces created via `create_surface()` share this connection and
    /// event queue — a single `dispatch()` call processes all events for all surfaces.
    pub fn new() -> Self {
        let conn = Connection::connect_to_env().unwrap();
        let (globals, event_queue) =
            registry_queue_init::<ShellState>(&conn).unwrap();
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

    /// Process one round of Wayland events (pointer motion, configure updates, etc.).
    /// Processes all pending events for all surfaces created on this state.
    pub fn dispatch(&mut self, state: &mut ShellState) {
        self.event_queue.roundtrip(state).unwrap();
    }

    /// Block until the given surface's initial `Configure` event arrives.
    /// Must be called *after* `surface.commit()` on the returned `WlSurface`.
    pub fn wait_for_configure(
        &mut self,
        state: &mut ShellState,
        surface_state: &Arc<Mutex<SurfaceState>>,
    ) {
        loop {
            {
                let ss = surface_state.lock().unwrap();
                if ss.configured {
                    break;
                }
            }
            self.event_queue.blocking_dispatch(state).unwrap();
        }
    }
}

// ==================== LAYER SURFACE ====================

/// A single Wayland layer surface — the canvas that the renderer draws onto.
///
/// All surfaces share the same `WaylandState` (connection + event queue).
/// Each has its own `SurfaceState` (dimensions) populated by the compositor's
/// Configure event.
///
/// The `WlSurface` is returned separately from `new()` so it can be
/// moved into the `Renderer` without partially moving `self`.
pub struct LayerSurface {
    /// The layer surface proxy — configure anchor, size, margins, etc. on this.
    pub layer_surface: ZwlrLayerSurfaceV1,
    /// Per-surface state (configured, width, height) — populated by the
    /// compositor's Configure event via the Dispatch impl.
    pub surface_state: Arc<Mutex<SurfaceState>>,
}

impl LayerSurface {
    /// Read the compositor's actual dimensions after `wait_for_configure`.
    pub fn dimensions(&self) -> (i32, i32) {
        let ss = self.surface_state.lock().unwrap();
        (ss.width, ss.height)
    }

    /// Create a new layer surface on the shared `WaylandState`.
    ///
    /// The caller must:
    /// 1. Configure `layer_surface` (anchor, size, exclusive zone, etc.)
    /// 2. Call `surface.commit()` on the returned `WlSurface`
    /// 3. Call `wl.wait_for_configure(state, &surface_state)` to block
    ///    until the compositor responds with actual dimensions
    ///
    /// Returns `(Self, WlSurface)` — the surface is separate so it can be
    /// moved into the `Renderer` without conflict.
    pub fn new(wl: &WaylandState, namespace: &str, surface_id: SurfaceId) -> (Self, WlSurface) {
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
            Layer::Top,
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
