mod canvas;
mod components;
mod hyprland;
mod renderer;
mod services;
mod shell;
mod ui;

use std::sync::Arc;

use crate::components::bar;
use crate::hyprland::HyprlandCompositor;
use crate::shell::compositor::Compositor;
use crate::shell::runtime::Shell;

// ==================== MAIN ====================

fn main() {
    // ---- Compositor backend ----
    let compositor: Arc<dyn Compositor> = Arc::new(HyprlandCompositor::new());
    let mut shell = Shell::new(compositor);

    bar::mount(&mut shell);

    // ---- Render loop (never returns) ----
    shell.run();
}
