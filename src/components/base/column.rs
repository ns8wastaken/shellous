use crate::components::arena::Slot;

pub struct ColumnNode {
    pub children: Vec<Slot>,
    pub spacing: f32,
}

impl ColumnNode {
    pub fn new(children: Vec<Slot>, spacing: f32) -> Self {
        Self { children, spacing }
    }
}
