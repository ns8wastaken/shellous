mod left;
mod middle;
mod workspace_dot;

pub(super) const BAR_HEIGHT: f32 = 30.0;
pub(super) const CORNER_RADIUS: f32 = 12.0;

use crate::components::layout::group::Group;
use crate::shell::layer_surface::{ShellAnchor, ShellLayer};
use crate::shell::runtime::{LayerSpec, Shell, SurfaceSpec};

use left::LeftPanel;
use middle::MiddlePanel;

pub fn mount(shell: &mut Shell) {
    let offset = 18;

    shell.mount(SurfaceSpec::Layer(LayerSpec {
        namespace: "shellous:bar".into(),
        anchor: ShellAnchor::TOP | ShellAnchor::LEFT | ShellAnchor::RIGHT,
        width: 0,
        height: BAR_HEIGHT as i32 + offset,
        exclusive_zone: BAR_HEIGHT as i32,
        layer: ShellLayer::Top,
        root: Some(Box::new(Group::new(vec![
            Box::new(LeftPanel::new(offset as f32)),
            Box::new(MiddlePanel::default()),
        ]))),
    }));
}
