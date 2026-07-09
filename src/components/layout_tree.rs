use crate::components::rect::Rect;

#[derive(Clone, Debug)]
pub struct LayoutNode {
    pub rect: Rect,
    pub children: Vec<LayoutNode>,
}

impl LayoutNode {
    pub fn new(rect: Rect) -> Self {
        Self { rect, children: Vec::new() }
    }
}
