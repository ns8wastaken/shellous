use crate::components::canvas::Rect;
use crate::renderer::programs::rect::RectStyle;

// ==================== DRAW COMMAND ====================

pub struct DrawCommand {
    pub rect: Rect,
    pub style: RectStyle,
}

// ==================== DRAW BATCH ====================

#[derive(Default)]
pub struct DrawBatch {
    commands: Vec<DrawCommand>,
}

impl DrawBatch {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, rect: Rect, style: &RectStyle) {
        if rect.w <= 0.0 || rect.h <= 0.0 {
            return;
        }
        self.commands.push(DrawCommand {
            rect,
            style: style.clone(),
        });
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }
}
