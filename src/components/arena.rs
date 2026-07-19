#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Slot {
    pub index: u32,
    pub generation: u32,
}

struct ArenaEntry<T> {
    value: Option<T>,
    generation: u32,
}

pub struct Arena<T> {
    entries: Vec<ArenaEntry<T>>,
    free_list: Vec<u32>,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            free_list: Vec::new(),
        }
    }

    pub fn insert(&mut self, value: T) -> Slot {
        if let Some(index) = self.free_list.pop() {
            let entry = &mut self.entries[index as usize];
            entry.generation += 1;
            entry.value = Some(value);
            Slot {
                index,
                generation: entry.generation,
            }
        } else {
            let index = self.entries.len() as u32;
            self.entries.push(ArenaEntry {
                value: Some(value),
                generation: 0,
            });
            Slot {
                index,
                generation: 0,
            }
        }
    }

    pub fn get(&self, slot: Slot) -> Option<&T> {
        self.entries
            .get(slot.index as usize)
            .filter(|e| e.generation == slot.generation)
            .and_then(|e| e.value.as_ref())
    }

    pub fn get_mut(&mut self, slot: Slot) -> Option<&mut T> {
        self.entries
            .get_mut(slot.index as usize)
            .filter(|e| e.generation == slot.generation)
            .and_then(|e| e.value.as_mut())
    }

    pub fn remove(&mut self, slot: Slot) -> Option<T> {
        let entry = self.entries.get_mut(slot.index as usize)?;
        if entry.generation == slot.generation && entry.value.is_some() {
            self.free_list.push(slot.index);
            entry.value.take()
        } else {
            None
        }
    }
}

// ponytail: slots never stale in practice (tree is static); add generation-fail
// handling if dynamic removal is introduced.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut arena = Arena::new();
        let s = arena.insert("hello");
        assert_eq!(arena.get(s), Some(&"hello"));
    }

    #[test]
    fn remove_frees_slot() {
        let mut arena = Arena::new();
        let s = arena.insert("a");
        arena.remove(s);
        assert_eq!(arena.get(s), None);
    }

    #[test]
    fn free_list_reuses_index() {
        let mut arena = Arena::new();
        let a = arena.insert("a");
        let b = arena.insert("b");
        arena.remove(a);
        let c = arena.insert("c");
        assert_eq!(c.index, a.index);
        assert_ne!(c.generation, a.generation);
        assert_eq!(arena.get(b), Some(&"b"));
        assert_eq!(arena.get(c), Some(&"c"));
        assert_eq!(arena.get(a), None); // stale slot
    }

    #[test]
    fn stale_generation_returns_none() {
        let mut arena = Arena::new();
        let s = arena.insert("x");
        let stale = Slot {
            index: s.index,
            generation: 99,
        };
        assert_eq!(arena.get(stale), None);
    }

    #[test]
    fn get_mut_modifies() {
        let mut arena = Arena::new();
        let s = arena.insert(42);
        *arena.get_mut(s).unwrap() = 100;
        assert_eq!(arena.get(s), Some(&100));
    }
}
