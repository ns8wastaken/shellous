use chrono::{DateTime, Local};

use crate::components::arena::Slot;
use crate::components::base::group::Group;
use crate::components::base::rect::RectNode;
use crate::components::base::text::TextNode;
use crate::components::geom::Size;
use crate::components::ui::{Controller, ElementArena, Node};
use crate::renderer::animation::cache::AnimationCache;
use crate::renderer::programs::rect::{CornerShape, RectStyle};
use crate::renderer::types::Color;
use crate::shell::event::ShellEvent;

const BAR_HEIGHT: f32 = 30.0;
const CORNER_RADIUS: f32 = BAR_HEIGHT * 0.5;

pub struct MiddlePanelController {
    text_slot: Slot,
    time: DateTime<Local>,
    time_formatted: String,
}

impl MiddlePanelController {
    pub fn mount(arena: &mut ElementArena) -> (Slot, Self) {
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

        let text = arena.insert(Node::Text(TextNode::new("", 14.0, Color::rgb(1.0, 1.0, 1.0))));
        let bg = arena.insert(Node::Rect(RectNode::new(Size::new(260.0, BAR_HEIGHT), base_style.fill(0.085, 0.095, 0.110, 1.0))));
        let root = arena.insert(Node::Group(Group::new(vec![bg, text])));

        let ctrl = MiddlePanelController {
            text_slot: text,
            time: DateTime::default(),
            time_formatted: String::new(),
        };

        (root, ctrl)
    }
}

impl Controller for MiddlePanelController {
    fn update(
        &mut self,
        event: &ShellEvent,
        _now: f32,
        _cache: &mut AnimationCache,
        arena: &mut ElementArena,
    ) -> bool {
        if let ShellEvent::ClockUpdated(snapshot) = event {
            self.time = snapshot.time;
            self.time_formatted = self.time.format("%H:%M").to_string();
            if let Some(node) = arena.get_mut(self.text_slot) {
                if let Node::Text(t) = node {
                    t.text = self.time_formatted.clone();
                }
            }
            true
        } else {
            false
        }
    }

    fn sync(
        &self,
        _now: f32,
        _cache: &AnimationCache,
        _arena: &mut ElementArena,
        _surface_w: f32,
        _surface_h: f32,
    ) {
    }
}
