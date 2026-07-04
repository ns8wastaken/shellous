mod left;
mod middle;

use crate::services::workspace::WorkspaceHandle;
use crate::shell::layer_surface::{ShellAnchor, ShellLayer};
use crate::shell::runtime::{Shell, SurfaceSpec};

use left::LeftPanel;
use middle::MiddlePanel;

fn surface(handle: WorkspaceHandle) -> SurfaceSpec {
    SurfaceSpec {
        namespace: "shellous:bar".into(),
        anchor: ShellAnchor::TOP | ShellAnchor::LEFT | ShellAnchor::RIGHT,
        width: 0,
        height: 36 + 18,
        exclusive_zone: 36,
        layer: ShellLayer::Top,
        elements: vec![
            Box::new(LeftPanel::new(handle)),
            Box::new(MiddlePanel::default()),
        ],
    }
}

pub fn mount(shell: &mut Shell) {
    let handle = shell.workspace().handle();
    shell.mount(surface(handle));
}
