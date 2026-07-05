/// A list of keyed items that can be reconciled against a new set of keys.
///
/// Useful for widget trees that need to add, remove, or reorder children
/// while preserving existing state (e.g., animated properties) for items
/// that already exist.
pub struct KeyedList<K: Eq + Copy, T> {
    pub items: Vec<(K, T)>,
}

impl<K: Eq + Copy, T> KeyedList<K, T> {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Reconcile against `ids`, preserving existing items by key and
    /// creating new ones via `factory` for unmatched keys. Order follows `ids`.
    pub fn reconcile(&mut self, ids: &[K], mut factory: impl FnMut(K) -> T) {
        let mut old = std::mem::take(&mut self.items);
        self.items = ids
            .iter()
            .map(|&id| match old.iter().position(|(k, _)| *k == id) {
                Some(pos) => old.remove(pos),
                None => (id, factory(id)),
            })
            .collect();
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter().map(|(_, v)| v)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.items.iter_mut().map(|(_, v)| v)
    }
}
