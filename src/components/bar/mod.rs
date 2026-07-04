mod left;
mod middle;

use crate::components::group::Group;
use crate::shell::layer_surface::{ShellAnchor, ShellLayer};
use crate::shell::runtime::{LayerSpec, Shell, SurfaceSpec};

use left::LeftPanel;
use middle::MiddlePanel;

pub fn mount(shell: &mut Shell) {
    let handle = shell.workspace().handle();

    shell.mount(SurfaceSpec::Layer(LayerSpec {
        namespace: "shellous:bar".into(),
        anchor: ShellAnchor::TOP | ShellAnchor::LEFT | ShellAnchor::RIGHT,
        width: 0,
        height: 36 + 18,
        exclusive_zone: 36,
        layer: ShellLayer::Top,
        root: Some(Box::new(Group::new(vec![
            Box::new(LeftPanel::new(handle)),
            Box::new(MiddlePanel::default()),
        ]))),
    }));
}
