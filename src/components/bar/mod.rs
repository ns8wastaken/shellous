mod left;
mod middle;
mod state;

use std::sync::Arc;

use crate::shell::layer_surface::{ShellAnchor, ShellLayer};
use crate::shell::runtime::{Shell, SurfaceSpec};

pub use state::{BarState, Workspace};
use left::LeftPanel;
use middle::MiddlePanel;

fn surface() -> SurfaceSpec {
    SurfaceSpec {
        namespace: "shellous:bar".into(),
        anchor: ShellAnchor::TOP | ShellAnchor::LEFT | ShellAnchor::RIGHT,
        width: 0,
        height: 36 + 18,
        exclusive_zone: 36,
        layer: ShellLayer::Top,
        elements: vec![
            Box::new(LeftPanel::default()),
            Box::new(MiddlePanel::default()),
        ],
    }
}

pub fn mount(shell: &mut Shell) {
    let state = Arc::clone(shell.bar_state());
    let compositor = Arc::clone(shell.compositor());
    compositor.refresh_bar(&state);
    compositor.spawn_event_listener(state);
    shell.mount(surface());
}
