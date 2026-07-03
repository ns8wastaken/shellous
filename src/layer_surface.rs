use wayland_client::{
    Connection, EventQueue,
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

use crate::display::AppState;

/// A Wayland layer surface — the canvas that the renderer draws onto.
///
/// `new()` connects to Wayland, binds globals, and creates the surface +
/// layer surface proxies. The caller is responsible for configuring
/// anchor, size, exclusive zone, etc. on `layer_surface`, then calling
/// `wait_for_configure()` to block until the compositor responds.
///
/// The `WlSurface` is returned separately from `new()` so it can be
/// moved into the `Renderer` without partially moving `self`.
pub struct LayerSurface {
    pub conn: Connection,
    /// The layer surface proxy — configure anchor, size, margins, etc. on this.
    pub layer_surface: ZwlrLayerSurfaceV1,
    #[allow(dead_code)]
    seat: WlSeat,
    event_queue: EventQueue<AppState>,
}

impl LayerSurface {
    /// Connect to Wayland, bind globals, and create a surface + layer surface.
    ///
    /// The caller must:
    /// 1. Configure `layer_surface` (anchor, size, exclusive zone, etc.)
    /// 2. Call `surface.commit()` on the returned `WlSurface`
    /// 3. Call `wait_for_configure()` to block until the compositor responds
    ///
    /// Returns `(Self, WlSurface)` — the surface is separate so it can be
    /// moved into the `Renderer` without conflict.
    pub fn new(namespace: &str) -> (Self, WlSurface) {
        let conn = Connection::connect_to_env().unwrap();
        let (globals, event_queue) =
            registry_queue_init::<AppState>(&conn).unwrap();
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

        let surface = wl_compositor.create_surface(&qh, ());

        let layer_surface = layer_shell.get_layer_surface(
            &surface,
            None,
            Layer::Top,
            namespace.to_string(),
            &qh,
            (),
        );

        (
            Self {
                conn,
                layer_surface,
                seat,
                event_queue,
            },
            surface,
        )
    }

    /// Block until the compositor sends the initial `Configure` event.
    /// Must be called *after* `surface.commit()` — the compositor responds
    /// to the commit with a configure that sets the actual dimensions.
    pub fn wait_for_configure(&mut self, state: &mut AppState) {
        while !state.configured {
            self.event_queue.blocking_dispatch(state).unwrap();
        }
    }

    /// Process one round of Wayland events (pointer motion, configure updates, etc.).
    /// Must be called every frame before rendering.
    pub fn dispatch(&mut self, state: &mut AppState) {
        self.event_queue.roundtrip(state).unwrap();
    }
}
