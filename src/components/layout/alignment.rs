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
            Alignment::Fill => parent,
            Alignment::Center => parent.place_center(child),
            Alignment::TopCenter => parent.place_top_center(child),
            Alignment::Start => parent.place_left_center(child),
            Alignment::End => parent.place_right_center(child),
        }
    }
}

// ==================== ALIGNMENT HELPERS ====================

pub fn align_center(parent: Rect, child: Size) -> Rect {
    parent.place_center(child)
}

pub fn align_left_center(parent: Rect, child: Size) -> Rect {
    parent.place_left_center(child)
}

pub fn align_right_center(parent: Rect, child: Size) -> Rect {
    parent.place_right_center(child)
}

pub fn align_top_left(parent: Rect, child: Size) -> Rect {
    parent.place_top_left(child)
}

pub fn align_top_center(parent: Rect, child: Size) -> Rect {
    parent.place_top_center(child)
}

pub fn align_bottom_left(parent: Rect, child: Size) -> Rect {
    parent.place_bottom_left(child)
}
