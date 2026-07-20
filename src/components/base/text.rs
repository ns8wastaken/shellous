use crate::renderer::types::Color;

pub struct TextNode {
    pub text: String,
    pub font_size: f32,
    pub color: Color,
}

impl TextNode {
    pub fn new(text: impl Into<String>, font_size: f32, color: Color) -> Self {
        Self { text: text.into(), font_size, color }
    }
}
