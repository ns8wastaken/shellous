use crate::components::canvas::{DrawingSurface, TranslatedCanvas};
use crate::services::workspace::WorkspaceSnapshot;
use crate::components::ui::{Element, RenderContext};

// ==================== PADDING ====================

/// A container that insets a single child element by `left`/`top`/`right`/`bottom`.
///
/// The child is shifted by `(left, top)` during drawing and click-hit testing,
/// and `size()` adds the inset dimensions to the child's natural size.
pub struct Padding {
    pub child: Box<dyn Element>,
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl Padding {
    pub fn new(child: Box<dyn Element>) -> Self {
        Self {
            child,
            left: 0.0,
            right: 0.0,
            top: 0.0,
            bottom: 0.0,
        }
    }

    pub fn left(mut self, v: f32) -> Self {
        self.left = v;
        self
    }

    pub fn right(mut self, v: f32) -> Self {
        self.right = v;
        self
    }

    pub fn top(mut self, v: f32) -> Self {
        self.top = v;
        self
    }

    pub fn bottom(mut self, v: f32) -> Self {
        self.bottom = v;
        self
    }

    pub fn all(mut self, v: f32) -> Self {
        self.left = v;
        self.right = v;
        self.top = v;
        self.bottom = v;
        self
    }

    pub fn x(mut self, v: f32) -> Self {
        self.left = v;
        self.right = v;
        self
    }

    pub fn y(mut self, v: f32) -> Self {
        self.top = v;
        self.bottom = v;
        self
    }
}

impl Element for Padding {
    fn update(&mut self, snapshot: &WorkspaceSnapshot) {
        self.child.update(snapshot);
    }

    fn tick_animations(&mut self, absolute_time: f32) -> bool {
        self.child.tick_animations(absolute_time)
    }

    fn draw(&self, surface: &dyn DrawingSurface, ctx: &RenderContext) {
        let tc = TranslatedCanvas::new(surface, self.left, self.top);
        self.child.draw(&tc, ctx);
    }

    fn size(&self) -> (f32, f32) {
        let (cw, ch) = self.child.size();
        (cw + self.left + self.right, ch + self.top + self.bottom)
    }

    fn on_click(&self, x: f32, y: f32, ctx: &RenderContext) -> bool {
        self.child.on_click(x - self.left, y - self.top, ctx)
    }

    fn sync_children(&mut self, ids: &[i32], factory: &mut dyn FnMut(i32) -> Box<dyn Element>) {
        self.child.sync_children(ids, factory);
    }
}
