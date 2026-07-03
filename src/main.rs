mod bar;
mod compositor;
mod display;
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
use crate::display::{ShellState, SurfaceState};
use crate::hyprland::HyprlandCompositor;
use crate::layer_surface::LayerSurface;
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

    // ---- Wayland canvas (layer surface) ----
    let surface_state = Arc::new(Mutex::new(SurfaceState::new(1920, 36)));

    let mut state = ShellState {
        bar: bar_state,
        compositor,
        pointer_pos: None,
    };

    let (mut bar_surface, surface) = LayerSurface::new(
        "shellous:bar",
        surface_state.clone()
    );

    // Configure the bar_surface: full-width top bar, 36px tall, exclusive zone
    bar_surface.layer_surface.set_anchor(Anchor::Top | Anchor::Left | Anchor::Right);
    bar_surface.layer_surface.set_size(0, 36);
    bar_surface.layer_surface.set_exclusive_zone(36);
    surface.commit();

    // Wait for the compositor to respond with the actual dimensions
    bar_surface.wait_for_configure(&mut state);

    // Read the actual dimensions from SurfaceState (set by the Configure event)
    let (actual_w, actual_h) = {
        let ss = surface_state.lock().unwrap();
        (ss.width, ss.height)
    };

    // ---- Renderer ----
    let panels: Vec<Box<dyn Panel>> = vec![Box::new(BarPanel::default())];
    let renderer = Renderer::new(
        &bar_surface.conn,
        surface,
        actual_w,
        actual_h,
        panels,
    );

    // ---- Render loop ----
    // TODO: limit fps
    loop {
        bar_surface.dispatch(&mut state);
        renderer.render_frame(&state);
    }
}
