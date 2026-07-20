use crate::components::geom::Rect;
use crate::renderer::programs::rect::RectStyle;
use crate::renderer::programs::text::TextStyle;

// ==================== SHAPE ====================

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Shape {
    #[default]
    Rect,
    Text,
}

// ==================== DRAW COMMAND ====================

pub enum DrawParams {
    Rect(RectStyle),
    Text(TextStyle),
}

impl DrawParams {
    /// Helper to map the unified parameter bundle back to your
    /// registry's sorting token / shape identifier.
    pub fn shape(&self) -> Shape {
        match self {
            DrawParams::Rect(_) => Shape::Rect,
            DrawParams::Text(_) => Shape::Text,
        }
    }
}

pub struct DrawCommand {
    pub rect: Rect,
    pub params: DrawParams,
}

// ==================== DRAW BATCH ====================

#[derive(Default)]
pub struct DrawBatch {
    commands: Vec<DrawCommand>,
}

impl DrawBatch {
    pub fn new() -> Self {
        Self { commands: Vec::with_capacity(64) }
    }

    pub fn push(&mut self, rect: Rect, params: DrawParams) {
        self.commands.push(DrawCommand { rect, params });
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }

    /// Sort commands by shape so that identical shapes are contiguous.
    /// Enables the renderer to dispatch whole groups to the matching
    /// program with minimal shader switches.
    pub fn sort_by_shape(&mut self) {
        self.commands.sort_by_key(|cmd| cmd.params.shape());
    }

    /// Iterate over groups of consecutive commands with the same shape.
    /// Requires `sort_by_shape()` to have been called first.
    pub fn shape_groups(&self) -> ShapeGroups<'_> {
        ShapeGroups {
            commands: &self.commands,
            idx: 0,
        }
    }
}

// ==================== SHAPE GROUPS ITERATOR ====================

pub struct ShapeGroups<'a> {
    commands: &'a [DrawCommand],
    idx: usize,
}

impl<'a> Iterator for ShapeGroups<'a> {
    type Item = (Shape, &'a [DrawCommand]);

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.commands.len() {
            return None;
        }
        let shape = self.commands[self.idx].params.shape();
        let start = self.idx;
        while self.idx < self.commands.len() && self.commands[self.idx].params.shape() == shape {
            self.idx += 1;
        }
        Some((shape, &self.commands[start..self.idx]))
    }
}
