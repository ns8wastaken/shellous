use crate::components::arena::Slot;

pub struct RowNode {
    pub children: Vec<Slot>,
    pub spacing: f32,
}

impl RowNode {
    pub fn new(children: Vec<Slot>, spacing: f32) -> Self {
        Self { children, spacing }
    }
}
