use crate::components::arena::Slot;
use crate::components::base::alignment::Alignment;

// ponytail: Align stores child as a Slot — the child lives in the per-surface
// arena, not as an owned trait object.

pub struct Align {
    pub child: Slot,
    pub alignment: Alignment,
}

impl Align {
    pub fn new(child: Slot, alignment: Alignment) -> Self {
        Self { child, alignment }
    }
}
