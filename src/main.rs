mod bar;
mod compositor;
mod display;
mod hyprland;
mod renderer;
mod wayland;

use wayland_client::{
    Connection,
    globals::registry_queue_init,
    protocol::{
        wl_compositor::WlCompositor,
        wl_seat::WlSeat,
    },
};

use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::Anchor,
};

use std::sync::{Arc, Mutex};

use crate::bar::BarState;
use crate::compositor::Compositor;
use crate::display::AppState;
use crate::hyprland::HyprlandCompositor;
use crate::renderer::Renderer;

// ==================== MAIN ====================

fn main() {
    // ---- Compositor backend (shared single instance) ----
    let compositor: Arc<dyn Compositor> = Arc::new(HyprlandCompositor::new());

    let bar_state = Arc::new(Mutex::new(BarState {
        workspaces: Vec::new(),
        active_id: -1,
    }));
    compositor.refresh_bar(&bar_state);
    compositor.clone().spawn_event_listener(bar_state.clone());

    // ---- Wayland display backend ----
    let conn = Connection::connect_to_env().unwrap();
    let (globals, mut event_queue) = registry_queue_init::<AppState>(&conn).unwrap();
    let qh = event_queue.handle();

    let mut state = AppState {
        configured: false,
        width: 1920,
        height: 36,
        pointer_pos: None,
        bar: bar_state.clone(),
        compositor: compositor.clone(),
    };

    let wl_compositor = globals
        .bind::<WlCompositor, _, _>(&qh, 1..=5, ())
        .expect("wl_compositor not available");
    let layer_shell = globals
        .bind::<ZwlrLayerShellV1, _, _>(&qh, 1..=4, ())
        .expect("zwlr_layer_shell_v1 not available");
    let seat = globals
        .bind::<WlSeat, _, _>(&qh, 1..=8, ())
        .expect("wl_seat not available");
    let _ = &seat;

    let surface = wl_compositor.create_surface(&qh, ());

    let layer_surface =
        layer_shell.get_layer_surface(&surface, None, Layer::Top, "rust-bar".into(), &qh, ());

    layer_surface.set_anchor(Anchor::Top | Anchor::Left | Anchor::Right);
    layer_surface.set_size(0, 36);
    layer_surface.set_exclusive_zone(36);

    surface.commit();

    while !state.configured {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }

    // ---------------- RENDERER ----------------
    let renderer = Renderer::new(&conn, surface, state.width, state.height);

    // ---------------- RENDER LOOP ----------------
    loop {
        event_queue.roundtrip(&mut state).unwrap();
        renderer.render_frame(&state);
    }
}
