mod bar;
mod compositor;
mod shell_state;
mod hyprland;
mod layer_surface;
mod renderer;
mod wayland;

use std::sync::{Arc, Mutex};

use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_surface_v1::Anchor,
};

use crate::bar::BarState;
use crate::compositor::Compositor;
use crate::shell_state::ShellState;
use crate::hyprland::HyprlandCompositor;
use crate::layer_surface::{LayerSurface, WaylandState};
use crate::renderer::panel::bar::BarPanel;
use crate::renderer::panel::Panel;
use crate::renderer::Renderer;

// ==================== MAIN ====================

fn main() {
    // ---- Compositor backend ----
    let compositor: Arc<dyn Compositor> = Arc::new(HyprlandCompositor::new());

    let bar_state = Arc::new(Mutex::new(BarState {
        workspaces: Vec::new(),
        active_id: -1,
    }));
    compositor.refresh_bar(&bar_state);
    compositor.clone().spawn_event_listener(bar_state.clone());

    // ---- Wayland connection (shared by all surfaces) ----
    let mut wl = WaylandState::new();

    let mut state = ShellState {
        bar: bar_state,
        compositor,
        pointer_pos: None,
        pointer_surface_height: 0.0,
    };

    // ---- Bar surface ----
    let (bar_surface, surface) = LayerSurface::new(&wl, "shellous:bar");

    bar_surface.layer_surface.set_anchor(Anchor::Top | Anchor::Left | Anchor::Right);
    bar_surface.layer_surface.set_size(0, 36 + 18);
    bar_surface.layer_surface.set_exclusive_zone(36);
    surface.commit();

    wl.wait_for_configure(&mut state, &bar_surface.surface_state);

    let (bar_w, bar_h) = bar_surface.dimensions();

    // ---- Renderer ----
    let panels: Vec<Box<dyn Panel>> = vec![Box::new(BarPanel::default())];
    let renderer = Renderer::new(
        &wl.conn,
        surface,
        bar_w,
        bar_h,
        panels,
    );

    // ---- Render loop ----
    // TODO: limit fps
    loop {
        wl.dispatch(&mut state);
        renderer.render_frame(&state);
    }
}
