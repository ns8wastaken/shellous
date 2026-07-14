use crate::components::rect::{Rect, Size};

// ==================== ALIGNMENT ====================

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Alignment {
    #[default]
    Fill,
    Center,
    TopCenter,
    Start,
    End,
}

impl Alignment {
    pub fn apply(self, parent: Rect, child: Size) -> Rect {
        match self {
            Alignment::Fill      => parent,
            Alignment::Center    => parent.place_center(child),
            Alignment::TopCenter => parent.place_top_center(child),
            Alignment::Start     => parent.place_left_center(child),
            Alignment::End       => parent.place_right_center(child),
        }
    }
}


