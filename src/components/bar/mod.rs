mod left;
mod middle;

use crate::components::layout::group::Group;
use crate::services::hyprland::get_option_int;
use crate::shell::layer_surface::{ShellAnchor, ShellLayer};
use crate::shell::runtime::{LayerSpec, Shell, SurfaceSpec};

use left::LeftPanel;
use middle::MiddlePanel;

pub fn mount(shell: &mut Shell) {
    let handle = shell.workspace().handle();
    let rounding = get_option_int("decoration:rounding").unwrap_or(0);
    let gaps_out = get_option_int("general:gaps_out").unwrap_or(0);
    let offset = (rounding + gaps_out) as f32;

    shell.mount(SurfaceSpec::Layer(LayerSpec {
        namespace: "shellous:bar".into(),
        anchor: ShellAnchor::TOP | ShellAnchor::LEFT | ShellAnchor::RIGHT,
        width: 0,
        height: 30 + offset as i32,
        exclusive_zone: 30,
        layer: ShellLayer::Top,
        root: Some(Box::new(Group::new(vec![
            Box::new(LeftPanel::new(handle, offset)),
            Box::new(MiddlePanel::default()),
        ]))),
    }));
}
