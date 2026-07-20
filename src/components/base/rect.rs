use crate::components::geom::Size;
use crate::renderer::programs::rect::RectStyle;

pub struct RectNode {
    pub style: RectStyle,
    pub size: Size,
    pub on_click: Option<Box<dyn Fn()>>,
}

impl RectNode {
    pub fn new(size: Size, style: RectStyle) -> Self {
        Self { style, size, on_click: None }
    }

    pub fn with_click(mut self, f: Box<dyn Fn()>) -> Self {
        self.on_click = Some(f);
        self
    }
}
