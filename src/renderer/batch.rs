use crate::components::rect::Rect;
use crate::renderer::programs::rect::RectStyle;

// ==================== SHAPE ====================

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Shape {
    #[default]
    Rect,
}

// ==================== DRAW COMMAND ====================

pub struct DrawCommand {
    pub rect: Rect,
    pub style: RectStyle,
    pub shape: Shape,
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

    pub fn push(&mut self, rect: Rect, style: RectStyle) {
        self.push_shape(rect, style, Shape::Rect);
    }

    pub fn push_shape(&mut self, rect: Rect, style: RectStyle, shape: Shape) {
        if rect.w <= 0.0 || rect.h <= 0.0 {
            return;
        }
        self.commands.push(DrawCommand {
            rect,
            style,
            shape,
        });
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
        self.commands.sort_by_key(|cmd| cmd.shape);
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
        let shape = self.commands[self.idx].shape;
        let start = self.idx;
        while self.idx < self.commands.len() && self.commands[self.idx].shape == shape {
            self.idx += 1;
        }
        Some((shape, &self.commands[start..self.idx]))
    }
}
