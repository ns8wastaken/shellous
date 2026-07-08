use crate::components::canvas::Rect;
use crate::renderer::programs::rect::RectStyle;

// ==================== SHAPE ====================

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Shape {
    #[default]
    Rect,
    Circle,
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

    pub fn push(&mut self, rect: Rect, style: &RectStyle) {
        self.push_shape(rect, style, Shape::Rect);
    }

    pub fn push_shape(&mut self, rect: Rect, style: &RectStyle, shape: Shape) {
        if rect.w <= 0.0 || rect.h <= 0.0 {
            return;
        }
        self.commands.push(DrawCommand {
            rect,
            style: style.clone(),
            shape,
        });
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }
}
