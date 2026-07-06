#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

mod components;
mod renderer;
mod services;
mod shell;

use std::sync::Arc;

use crate::components::bar;
use crate::services::hyprland::HyprlandCompositor;
use crate::shell::compositor::Compositor;
use crate::shell::runtime::Shell;

// ==================== MAIN ====================

fn main() {
    // ---- Compositor backend ----
    let compositor: Arc<dyn Compositor> = Arc::new(HyprlandCompositor::new());
    let mut shell = Shell::new(compositor);

    bar::mount(&mut shell);

    // ---- Tray listener (background thread, tokio runtime) ----
    crate::services::tray::spawn_tray_listener();

    // ---- Render loop (never returns) ----
    shell.run();
}
