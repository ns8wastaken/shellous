mod left_panel;
mod middle_panel;

use std::sync::Arc;
use left_panel::LeftPanelController;
use middle_panel::MiddlePanelController;

use crate::components::arena::Arena;
use crate::components::base::Alignment;
use crate::components::base::align::AlignNode;
use crate::components::base::group::GroupNode;
use crate::components::ui::{Controller, Node};
use crate::renderer::animation::cache::AnimationCache;
use crate::shell::compositor::Compositor;
use crate::shell::layer_surface::{ShellAnchor, ShellLayer};
use crate::shell::runtime::{LayerSpec, Shell, SurfaceSpec};

pub(super) const BAR_HEIGHT: f32 = 30.0;

pub fn mount(shell: &mut Shell, compositor: Arc<dyn Compositor>) {
    let offset = 18;
    let mut cache = AnimationCache::new();
    let mut arena: Arena<Node> = Arena::new();

    let (middle_root, middle_ctrl) = MiddlePanelController::mount(&mut arena);
    let align = arena.insert(Node::Align(AlignNode::new(middle_root, Alignment::TopCenter)));
    let (left_root, left_ctrl) = LeftPanelController::mount(
        compositor,
        offset as f32,
        &mut arena,
        &mut cache
    );
    let root = arena.insert(Node::Group(GroupNode::new(vec![left_root, align])));

    let controllers: Vec<Box<dyn Controller>> = vec![
        Box::new(middle_ctrl),
        Box::new(left_ctrl),
    ];

    shell.mount(SurfaceSpec::Layer(LayerSpec {
        namespace: "shellous:bar".into(),
        anchor: ShellAnchor::TOP | ShellAnchor::LEFT | ShellAnchor::RIGHT,
        width: 0,
        height: BAR_HEIGHT as i32 + offset,
        exclusive_zone: BAR_HEIGHT as i32,
        layer: ShellLayer::Top,
        root: Some(root),
    }), cache, arena, controllers);
}
