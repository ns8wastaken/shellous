use crate::components::arena::Slot;

pub struct Padding {
    pub child: Slot,
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Padding {
    pub fn new(child: Slot, insets: (f32, f32, f32, f32)) -> Self {
        Self {
            child,
            left: insets.0,
            top: insets.1,
            right: insets.2,
            bottom: insets.3,
        }
    }
}
