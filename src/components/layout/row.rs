use crate::components::canvas::{DrawingSurface, TranslatedCanvas};
use crate::components::ui::{Element, RenderContext};

// ==================== ROW ====================

/// A horizontally-arranged group of child elements with consistent spacing.
///
/// Children are drawn at `(cursor_x, 0)` where `cursor_x` advances by
/// `child.size().0 + self.spacing`. Position the row itself by wrapping it
/// in a `TranslatedCanvas` — Row always treats its own origin as (0, 0).
pub struct Row {
    pub children: Vec<Box<dyn Element>>,
    spacing: f32,
}

impl Row {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            spacing: 8.0,
        }
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn push(&mut self, child: Box<dyn Element>) {
        self.children.push(child);
    }
}

impl Element for Row {
    fn tick_animations(&mut self, absolute_time: f32) -> bool {
        let mut active = false;
        for child in &mut self.children {
            if child.tick_animations(absolute_time) {
                active = true;
            }
        }
        active
    }

    fn draw(&self, surface: &dyn DrawingSurface, ctx: &RenderContext) {
        let mut cx = 0.0;
        for child in &self.children {
            let tc = TranslatedCanvas::new(surface, cx, 0.0);
            child.draw(&tc, ctx);
            cx += child.size(ctx.absolute_time).0 + self.spacing;
        }
    }

    fn size(&self, absolute_time: f32) -> (f32, f32) {
        if self.children.is_empty() {
            return (0.0, 0.0);
        }
        let mut width: f32 = 0.0;
        for (i, c) in self.children.iter().enumerate() {
            width += c.size(absolute_time).0;
            if i + 1 < self.children.len() {
                width += self.spacing;
            }
        }
        let height = self
            .children
            .iter()
            .map(|c| c.size(absolute_time).1)
            .fold(0.0f32, f32::max);
        (width, height)
    }

    fn on_click(&self, x: f32, y: f32, ctx: &RenderContext) -> bool {
        let mut positions: Vec<f32> = Vec::with_capacity(self.children.len());
        let mut cx = 0.0;
        for c in &self.children {
            positions.push(cx);
            cx += c.size(ctx.absolute_time).0 + self.spacing;
        }
        for (i, child) in self.children.iter().enumerate().rev() {
            let px = positions[i];
            let (cw, ch) = child.size(ctx.absolute_time);
            if x >= px && x <= px + cw && y >= 0.0 && y <= ch {
                if child.on_click(x - px, y, ctx) {
                    return true;
                }
            }
        }
        false
    }

    fn replace_children(
        &mut self,
        children: Vec<Box<dyn Element>>,
    ) -> Vec<Box<dyn Element>> {
        std::mem::replace(&mut self.children, children)
    }

    fn push_child(&mut self, child: Box<dyn Element>) {
        self.children.push(child);
    }
}
