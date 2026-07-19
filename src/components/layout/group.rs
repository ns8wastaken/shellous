use crate::components::arena::Slot;

// ponytail: Group stores children as Vec<Slot> — children live in the per-surface
// arena, not as owned trait objects. Nodes are resolved through the arena during
// layout/draw/click passes.

pub struct Group {
    pub children: Vec<Slot>,
}

impl Group {
    pub fn new(children: Vec<Slot>) -> Self {
        Self { children }
    }
}
