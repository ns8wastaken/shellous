use chrono::{DateTime, Local};

use crate::components::layout_tree::LayoutNode;
use crate::components::rect::Size;
use crate::components::ui::{Element, RenderContext};
use crate::renderer::batch::{DrawBatch, DrawParams};
use crate::renderer::programs::rect::{
    CornerShape, RectStyle,
};
use crate::renderer::programs::text::TextStyle;
use crate::renderer::types::Color;
use crate::shell::event::ShellEvent;
use super::{BAR_HEIGHT, CORNER_RADIUS};

pub struct MiddlePanel {
    pub width: f32,
    time: DateTime<Local>,
    time_formatted: String,
}

impl Default for MiddlePanel {
    fn default() -> Self {
        Self {
            width: 260.0,
            time: DateTime::default(),
            time_formatted: String::new(),
        }
    }
}

impl Element for MiddlePanel {
    fn update(&mut self, event: &ShellEvent) -> bool {
        if let ShellEvent::ClockUpdated(snapshot) = event {
            self.time = snapshot.time;
            self.time_formatted = self.time.format("%H:%M").to_string(); // format lives here
            true
        } else {
            false
        }
    }

    fn layout(&self, _available: Size) -> Size {
        Size { w: self.width, h: BAR_HEIGHT }
    }

    fn draw(&self, node: &LayoutNode, batch: &mut DrawBatch, _ctx: &RenderContext) {
        let base_style = RectStyle::new()
            .corners(
                CornerShape::Concave,
                CornerShape::Concave,
                CornerShape::Convex,
                CornerShape::Convex,
            )
            .all_radius(CORNER_RADIUS)
            .inset_left(CORNER_RADIUS)
            .inset_right(CORNER_RADIUS);

        batch.push(
            node.rect,
            DrawParams::Rect(
                base_style
                    .clone()
                    .fill(0.0, 0.0, 0.0, 0.5)
                    .softness(10.0)
                    .shadow(0.0, 0.0)
            )
        );

        batch.push(
            node.rect,
            DrawParams::Rect(base_style.fill(0.085, 0.095, 0.110, 1.0)),
        );

        let mut text_rect = node.rect;

        batch.push(
            text_rect,
            DrawParams::Text(
                TextStyle::new()
                    .text(self.time_formatted.clone())
                    .size(14.0)
                    .color(Color::rgb(1.0, 1.0, 1.0))
            )
        );
    }
}
