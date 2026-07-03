mod components;
mod hyprland;
mod renderer;
mod shell;
mod ui;

use std::sync::{Arc, Mutex};

use crate::components::bar::{self, BarState};
use crate::hyprland::HyprlandCompositor;
use crate::shell::compositor::Compositor;
use crate::shell::runtime::Shell;

// ==================== MAIN ====================

fn main() {
    // ---- Compositor backend ----
    let compositor: Arc<dyn Compositor> = Arc::new(HyprlandCompositor::new());

    let bar_state = Arc::new(Mutex::new(BarState {
        workspaces: Vec::new(),
        active_id: -1,
    }));
    let mut shell = Shell::new(compositor.clone(), bar_state.clone());

    bar::install(&mut shell, compositor.clone(), bar_state);

    // ---- Render loop (never returns) ----
    shell.run();
}
