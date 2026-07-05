use crate::components::canvas::{DrawingSurface, TranslatedCanvas};
use crate::services::workspace::WorkspaceSnapshot;
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
}

impl Row {
    fn layout(&self) -> Vec<f32> {
        let mut cx = 0.0;
        let mut offsets = Vec::with_capacity(self.children.len());
        for c in &self.children {
            offsets.push(cx);
            cx += c.size().0 + self.spacing;
        }
        offsets
    }
}

impl Element for Row {
    fn update(&mut self, snapshot: &WorkspaceSnapshot) {
        for child in &mut self.children {
            child.update(snapshot);
        }
    }

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
        for (child, cx) in self.children.iter().zip(self.layout()) {
            let tc = TranslatedCanvas::new(surface, cx, 0.0);
            child.draw(&tc, ctx);
        }
    }

    fn size(&self) -> (f32, f32) {
        let offsets = self.layout();
        let width = match offsets.last() {
            Some(&last) => {
                let idx = offsets.len() - 1;
                last + self.children[idx].size().0
            }
            None => 0.0,
        };
        let height = self
            .children
            .iter()
            .map(|c| c.size().1)
            .fold(0.0f32, f32::max);
        (width, height)
    }

    fn on_click(&self, x: f32, y: f32, ctx: &RenderContext) -> bool {
        let offsets = self.layout();
        for (i, child) in self.children.iter().enumerate().rev() {
            let px = offsets[i];
            let (cw, ch) = child.size();
            if x >= px && x <= px + cw && y >= 0.0 && y <= ch {
                if child.on_click(x - px, y, ctx) {
                    return true;
                }
            }
        }
        false
    }

    fn sync_children(&mut self, ids: &[i32], factory: &mut dyn FnMut(i32) -> Box<dyn Element>) {
        let old = std::mem::take(&mut self.children);
        let mut by_id: Vec<(i32, Box<dyn Element>)> = old
            .into_iter()
            .filter_map(|c| c.id().map(|id| (id, c)))
            .collect();

        for id in ids {
            match by_id.iter().position(|(eid, _)| eid == id) {
                Some(idx) => self.children.push(by_id.remove(idx).1),
                None => self.children.push(factory(*id)),
            }
        }
    }
}
