pub(crate) mod left;
pub(crate) mod middle;

pub(super) const BAR_HEIGHT: f32 = 30.0;
pub(super) const CORNER_RADIUS: f32 = BAR_HEIGHT * 0.5;

use crate::components::arena::Arena;
use crate::components::layout::Alignment;
use crate::components::layout::align::Align;
use crate::components::layout::group::Group;
use crate::components::ui::Node;
use crate::renderer::animation::cache::AnimationCache;
use crate::shell::layer_surface::{ShellAnchor, ShellLayer};
use crate::shell::runtime::{LayerSpec, Shell, SurfaceSpec};

use left::LeftPanel;
use middle::MiddlePanel;

pub fn mount(shell: &mut Shell) {
    let offset = 18;
    let mut cache = AnimationCache::new();
    let mut arena: Arena<Node> = Arena::new();

    let middle = arena.insert(Node::MiddlePanel(MiddlePanel::default()));
    let align = arena.insert(Node::Align(Align::new(middle, Alignment::TopCenter)));
    let left = arena.insert(Node::LeftPanel(LeftPanel::new(offset as f32, &mut cache)));
    let root = arena.insert(Node::Group(Group::new(vec![left, align])));

    shell.mount(SurfaceSpec::Layer(LayerSpec {
        namespace: "shellous:bar".into(),
        anchor: ShellAnchor::TOP | ShellAnchor::LEFT | ShellAnchor::RIGHT,
        width: 0,
        height: BAR_HEIGHT as i32 + offset,
        exclusive_zone: BAR_HEIGHT as i32,
        layer: ShellLayer::Top,
        root: Some(root),
    }), cache, arena);
}
