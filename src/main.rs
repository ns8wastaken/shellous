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
use crate::services::workspace::WorkspaceService;
use crate::shell::compositor::Compositor;
use crate::shell::event::ShellModule;
use crate::shell::runtime::Shell;

fn main() {
    let compositor: Arc<dyn Compositor> = Arc::new(HyprlandCompositor::new());
    let mut shell = Shell::new(compositor.clone());

    let workspace_service = WorkspaceService::new(compositor);
    // let tray_service = TrayService::new(); // <-- Add future modules here

    bar::mount(&mut shell);

    let modules: Vec<Box<dyn ShellModule>> = vec![
        Box::new(workspace_service),
        // Box::new(tray_service); // <-- Add future service lifecycles here
    ];

    shell.run(modules);
}
