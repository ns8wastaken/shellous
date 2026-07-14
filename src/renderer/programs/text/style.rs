use crate::renderer::types::Color;

#[derive(Clone, Debug, Default)]
pub struct TextStyle {
    pub text: String,
    pub font_size: f32, // pixel size
    pub color: Color,
}

impl TextStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}
