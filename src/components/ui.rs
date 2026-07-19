use crate::components::arena::Arena;
use crate::components::arena::Slot;
use crate::components::bar::left::LeftPanel;
use crate::components::bar::middle::MiddlePanel;
use crate::components::layout::align::Align;
use crate::components::layout::group::Group;
use crate::components::layout_tree::LayoutNode;
use crate::components::rect::{Rect, Size};
use crate::renderer::animation::cache::AnimationCache;
use crate::renderer::batch::DrawBatch;
use crate::shell::event::ShellEvent;
use crate::shell::state::ShellState;

// ponytail: Node enum — closed set of widget types; add variants for new widgets.
// Swap to trait object if dynamic plugins needed.

pub type ElementArena = Arena<Node>;

pub enum Node {
    Group(Group),
    Align(Align),
    LeftPanel(LeftPanel),
    MiddlePanel(MiddlePanel),
}

impl Node {
    pub fn children(&self) -> &[Slot] {
        match self {
            Self::Group(g) => &g.children,
            Self::Align(a) => std::slice::from_ref(&a.child),
            Self::LeftPanel(_) | Self::MiddlePanel(_) => &[],
        }
    }

    pub fn update(&mut self, event: &ShellEvent, now: f32, cache: &mut AnimationCache) -> bool {
        match self {
            Self::Group(_) | Self::Align(_) => false,
            Self::LeftPanel(p) => p.update(event, now, cache),
            Self::MiddlePanel(p) => p.update(event),
        }
    }

    pub fn derive_targets(&self, now: f32, cache: &mut AnimationCache, arena: &ElementArena) {
        match self {
            Self::Group(g) => {
                for &slot in &g.children {
                    arena.get(slot).unwrap().derive_targets(now, cache, arena);
                }
            }
            Self::Align(a) => {
                arena.get(a.child).unwrap().derive_targets(now, cache, arena);
            }
            Self::LeftPanel(p) => p.derive_targets(now, cache),
            Self::MiddlePanel(_) => {}
        }
    }

    pub fn layout(&self, available: Size, cache: &AnimationCache, arena: &ElementArena) -> Size {
        match self {
            Self::Group(_) => available,
            Self::Align(a) => {
                let child = arena.get(a.child).unwrap();
                child.layout(available, cache, arena)
            }
            Self::LeftPanel(p) => p.layout(available, cache),
            Self::MiddlePanel(p) => p.layout(available),
        }
    }

    pub fn layout_tree(
        &self,
        rect: Rect,
        cache: &AnimationCache,
        arena: &ElementArena,
    ) -> LayoutNode {
        match self {
            Self::Group(g) => LayoutNode {
                rect,
                children: g
                    .children
                    .iter()
                    .map(|&slot| {
                        arena
                            .get(slot)
                            .unwrap()
                            .layout_tree(rect, cache, arena)
                    })
                    .collect(),
            },
            Self::Align(a) => {
                let child = arena.get(a.child).unwrap();
                let child_size = child.layout(rect.size(), cache, arena);
                let child_rect = a.alignment.apply(rect, child_size);
                LayoutNode {
                    rect,
                    children: vec![child.layout_tree(child_rect, cache, arena)],
                }
            }
            Self::LeftPanel(p) => p.layout_tree(rect, cache),
            Self::MiddlePanel(_) => LayoutNode::new(rect),
        }
    }

    pub fn draw(&self, node: &LayoutNode, batch: &mut DrawBatch, ctx: &RenderContext) {
        match self {
            Self::Group(g) => {
                for (&slot, child_node) in g.children.iter().zip(&node.children) {
                    ctx.arena.get(slot).unwrap().draw(child_node, batch, ctx);
                }
            }
            Self::Align(a) => {
                ctx.arena
                    .get(a.child)
                    .unwrap()
                    .draw(&node.children[0], batch, ctx);
            }
            Self::LeftPanel(p) => p.draw(node, batch, ctx),
            Self::MiddlePanel(p) => p.draw(node, batch, ctx),
        }
    }

    pub fn on_click(&self, node: &LayoutNode, x: f32, y: f32, ctx: &RenderContext) -> bool {
        match self {
            Self::Group(g) => {
                for (&slot, child_node) in g.children.iter().zip(&node.children).rev() {
                    if child_node.rect.contains(x, y)
                        && ctx
                            .arena
                            .get(slot)
                            .unwrap()
                            .on_click(child_node, x, y, ctx)
                    {
                        return true;
                    }
                }
                false
            }
            Self::Align(a) => ctx
                .arena
                .get(a.child)
                .unwrap()
                .on_click(&node.children[0], x, y, ctx),
            Self::LeftPanel(p) => p.on_click(node, x, y, ctx),
            Self::MiddlePanel(_) => false,
        }
    }
}

pub struct RenderContext<'a> {
    pub surface_w: f32,
    pub surface_h: f32,
    pub state: &'a ShellState,
    pub animations: &'a AnimationCache,
    pub arena: &'a ElementArena,
}
