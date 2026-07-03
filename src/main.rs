mod action;
mod bar;
mod compositor;
mod egl_state;
mod hyprland;
mod layer_surface;
mod managed_surface;
mod renderer;
mod shell;
mod shell_state;
mod surface_id;
mod wayland;

use std::sync::{Arc, Mutex};

use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::Anchor;

use crate::bar::BarState;
use crate::compositor::Compositor;
use crate::hyprland::HyprlandCompositor;
use crate::renderer::panel::bar::BarPanel;
use crate::shell::{Shell, SurfaceConfig};

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

    // ---- Shell ----
    let mut shell = Shell::new(compositor, bar_state);

    // ---- Bar surface ----
    shell.add_surface(SurfaceConfig {
        namespace: "shellous:bar".into(),
        anchor: Anchor::Top | Anchor::Left | Anchor::Right,
        width: 0,
        height: 36 + 18,
        exclusive_zone: 36,
        panels: vec![Box::new(BarPanel::default())],
    });

    // ---- Render loop (never returns) ----
    shell.run();
}
