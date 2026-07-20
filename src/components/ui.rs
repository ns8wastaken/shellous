use crate::components::arena::{Arena, Slot};
use crate::components::base::{AlignNode, GroupNode, PaddingNode, RectNode, RowNode, TextNode};
use crate::components::layout_tree::LayoutNode;
use crate::components::geom::{Rect, Size};
use crate::renderer::animation::cache::AnimationCache;
use crate::renderer::batch::DrawBatch;
use crate::shell::event::ShellEvent;
use crate::shell::state::ShellState;
use crate::components::base::Element;

pub type ElementArena = Arena<Node>;

#[enum_dispatch::enum_dispatch(Element)]
pub enum Node {
    // --------------------------------------------------------------------
    // VISUAL PRIMITIVES (Leaf nodes, map directly to Shader Render Passes)
    // --------------------------------------------------------------------
    /// Submits a quad to the GPU rect/instancing shader
    Rect(RectNode),
    /// Submits character glyphs to the text/font rendering engine
    Text(TextNode),

    // -------------------------------------------------------------------------
    // STRUCTURAL NODES / LAYOUTERS (Zero rendering overhead, position children)
    // -------------------------------------------------------------------------
    /// A pass-through layout that does not define any alignments
    Group(GroupNode),
    /// Spreads children out along the X axis with specific spacing
    Row(RowNode),
    /// Insets a single child node from its parent boundaries
    Padding(PaddingNode),
    /// Shifts a child based on alignment position (Center, TopRight, etc.)
    Align(AlignNode),
}

pub trait Controller {
    fn update(&mut self, event: &ShellEvent, now: f32, cache: &mut AnimationCache, arena: &mut ElementArena) -> bool;
    fn sync(&self, now: f32, cache: &AnimationCache, arena: &mut ElementArena, surface_w: f32, surface_h: f32);
    fn on_click(&self, x: f32, y: f32, arena: &ElementArena, layout: &LayoutNode, ctx: &RenderContext) -> bool {
        let _ = (x, y, arena, layout, ctx);
        false
    }
}

pub struct RenderContext<'a> {
    pub surface_w: f32,
    pub surface_h: f32,
    pub state: &'a ShellState,
    pub animations: &'a AnimationCache,
    pub arena: &'a ElementArena,
}
