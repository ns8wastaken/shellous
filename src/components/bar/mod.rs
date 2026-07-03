mod left;
mod middle;
mod state;

use std::sync::{Arc, Mutex};

use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::Layer, zwlr_layer_surface_v1::Anchor,
};

use crate::shell::compositor::Compositor;
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

pub fn install(shell: &mut Shell, compositor: Arc<dyn Compositor>, state: Arc<Mutex<BarState>>) {
    compositor.refresh_bar(&state);
    compositor.clone().spawn_event_listener(state.clone());
    shell.mount(surface());
}
