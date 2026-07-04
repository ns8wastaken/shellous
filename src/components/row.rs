use crate::canvas::{DrawingSurface, TranslatedCanvas};
use crate::ui::{Element, RenderContext};

// ==================== ROW ====================

/// A horizontally-arranged group of child elements with consistent spacing.
///
/// Each child is drawn at `(cursor_x, self.y)` where `cursor_x` advances
/// after each child by `child.size().0 + self.spacing`. Children receive a
/// `TranslatedCanvas` so they can draw at their own local origin — Row
/// composes the translation for them.
///
/// Click hits are dispatched to the child whose local bounding box contains
/// the click; click coordinates are translated back into child-local coords.
///
/// Builder:
/// ```ignore
/// Row::new()
///     .at(50.0, 18.0)
///     .spacing(8.0)
///     .add(Box::new(tag_a))
///     .add(Box::new(tag_b))
/// ```
pub struct Row {
    children: Vec<Box<dyn Element>>,
    spacing: f32,
    x: f32,
    y: f32,
}

impl Default for Row {
    fn default() -> Self {
        Self {
            children: Vec::new(),
            spacing: 8.0,
            x: 0.0,
            y: 0.0,
        }
    }
}

impl Row {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn add(mut self, child: Box<dyn Element>) -> Self {
        self.children.push(child);
        self
    }
}

impl Element for Row {
    fn draw(&self, surface: &dyn DrawingSurface, ctx: &RenderContext) {
        let mut cx = self.x;
        for child in &self.children {
            let tc = TranslatedCanvas::new(surface, cx, self.y);
            child.draw(&tc, ctx);
            cx += child.size().0 + self.spacing;
        }
    }

    fn size(&self) -> (f32, f32) {
        if self.children.is_empty() {
            return (0.0, 0.0);
        }
        let mut width: f32 = 0.0;
        for (i, c) in self.children.iter().enumerate() {
            width += c.size().0;
            if i + 1 < self.children.len() {
                width += self.spacing;
            }
        }
        let height = self
            .children
            .iter()
            .map(|c| c.size().1)
            .fold(0.0f32, f32::max);
        (width, height)
    }

    fn on_click(&self, x: f32, y: f32, ctx: &RenderContext) -> bool {
        // Precompute every child's left edge so we can dispatch topmost-first.
        let mut positions: Vec<f32> = Vec::with_capacity(self.children.len());
        let mut cx = self.x;
        for c in &self.children {
            positions.push(cx);
            cx += c.size().0 + self.spacing;
        }
        for (i, child) in self.children.iter().enumerate().rev() {
            let px = positions[i];
            let (cw, ch) = child.size();
            if x >= px && x <= px + cw && y >= self.y && y <= self.y + ch {
                if child.on_click(x - px, y - self.y, ctx) {
                    return true;
                }
            }
        }
        false
    }
}
