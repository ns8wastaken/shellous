use crate::components::layout_tree::LayoutNode;
use crate::components::rect::{Rect, Size};
use crate::renderer::animation::cache::AnimationCache;
use crate::renderer::batch::DrawBatch;
use crate::shell::event::ShellEvent;
use crate::shell::state::ShellState;

// ==================== RENDER CONTEXT ====================

pub struct RenderContext<'a> {
    pub surface_w: f32,
    pub surface_h: f32,
    pub state: &'a ShellState,
    pub animations: &'a AnimationCache,
}

// ==================== ELEMENT ====================

pub trait Element {
    fn update(&mut self, _event: &ShellEvent, _now: f32, _cache: &mut AnimationCache) -> bool {
        false
    }

    /// Re-derive chasing animation targets (e.g., panel width following dot widths).
    /// Called after `cache.tick()` when active animations exist, and after
    /// `update()` when state changed. Takes `&self` because elements don't mutate
    /// themselves here — only the cache.
    fn derive_targets(&self, _now: f32, _cache: &mut AnimationCache) {}

    /// Compute the element's desired size given available space.
    /// Called during the layout phase (CPU only).
    fn layout(&self, available: Size, cache: &AnimationCache) -> Size;

    /// Build the retained layout tree for a given parent rect.
    /// Default: leaf node (no children).
    fn layout_tree(&self, rect: Rect, _cache: &AnimationCache) -> LayoutNode {
        LayoutNode::new(rect)
    }

    /// Collect draw commands using the cached layout node.
    fn draw(&self, node: &LayoutNode, batch: &mut DrawBatch, ctx: &RenderContext);

    /// Walk the cached layout tree for hit-testing.
    /// Containers override to iterate their children; leaves override to handle clicks.
    fn on_click(&self, _node: &LayoutNode, _x: f32, _y: f32, _ctx: &RenderContext) -> bool {
        false
    }
}
