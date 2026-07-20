pub mod align;
pub mod alignment;
pub mod group;
pub mod padding;
pub mod rect;
pub mod row;
pub mod stacks;
pub mod text;

pub use align::AlignNode;
pub use alignment::Alignment;
pub use group::GroupNode;
pub use padding::PaddingNode;
pub use rect::RectNode;
pub use row::RowNode;
pub use stacks::stack_horizontal;
pub use text::TextNode;

use crate::components::arena::Slot;
use crate::components::geom::{Rect, Size};
use crate::components::layout_tree::LayoutNode;
use crate::components::ui::{ElementArena, RenderContext};
use crate::renderer::animation::cache::AnimationCache;
use crate::renderer::batch::DrawBatch;

#[enum_dispatch::enum_dispatch]
pub trait Element {
    fn children(&self) -> &[Slot];
    fn layout(&self, available: Size, cache: &AnimationCache, arena: &ElementArena) -> Size;
    fn layout_tree(&self, rect: Rect, cache: &AnimationCache, arena: &ElementArena) -> LayoutNode;
    fn draw(&self, node: &LayoutNode, batch: &mut DrawBatch, ctx: &RenderContext);
    fn on_click(&self, node: &LayoutNode, x: f32, y: f32, ctx: &RenderContext) -> bool;
}
