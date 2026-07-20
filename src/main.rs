#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

mod components;
mod renderer;
mod services;
mod shell;

use std::sync::Arc;

use crate::components::widgets::bar;
use crate::services::clock::ClockService;
use crate::services::hyprland::HyprlandCompositor;
use crate::services::workspace::WorkspaceService;
use crate::shell::compositor::Compositor;
use crate::shell::event::ShellModule;
use crate::shell::runtime::Shell;

fn main() {
    let compositor: Arc<dyn Compositor> = Arc::new(HyprlandCompositor::new());
    let mut shell = Shell::new(compositor.clone());

    bar::mount(&mut shell, compositor.clone());

    let modules: Vec<Box<dyn ShellModule>> = vec![
        Box::new(WorkspaceService::new(compositor)),
        Box::new(ClockService::new()),
        // Box::new(tray_service); // <-- Add future services here
    ];

    shell.run(modules);
}
