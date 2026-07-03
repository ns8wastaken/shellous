mod left;
mod middle;
mod state;

use std::sync::Arc;

use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::Layer, zwlr_layer_surface_v1::Anchor,
};

use crate::shell::runtime::{Shell, SurfaceSpec};
use crate::ui::SurfaceRole;

pub use left::LeftPanel;
pub use middle::MiddlePanel;
pub use state::{BarState, Workspace};

pub fn surface() -> SurfaceSpec {
    SurfaceSpec {
        namespace: "shellous:bar".into(),
        anchor: Anchor::Top | Anchor::Left | Anchor::Right,
        width: 0,
        height: 36 + 18,
        exclusive_zone: 36,
        layer: Layer::Top,
        role: SurfaceRole::Bar,
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
