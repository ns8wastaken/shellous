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
use crate::display::AppState;
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
    let mut state = AppState {
        configured: false,
        width: 1920,
        height: 36,
        pointer_pos: None,
        bar: bar_state,
        compositor,
    };

    let (mut canvas, surface) = LayerSurface::new("rust-bar");

    // Configure the canvas: full-width top bar, 36px tall, exclusive zone
    canvas.layer_surface.set_anchor(Anchor::Top | Anchor::Left | Anchor::Right);
    canvas.layer_surface.set_size(0, 36);
    canvas.layer_surface.set_exclusive_zone(36);
    surface.commit();

    // Wait for the compositor to respond with the actual dimensions
    canvas.wait_for_configure(&mut state);

    // ---- Renderer ----
    let panels: Vec<Box<dyn Panel>> = vec![Box::new(BarPanel::default())];
    let renderer = Renderer::new(
        &canvas.conn,
        surface,
        state.width,
        state.height,
        panels,
    );

    // ---- Render loop ----
    // TODO: limit fps
    loop {
        canvas.dispatch(&mut state);
        renderer.render_frame(&state);
    }
}
