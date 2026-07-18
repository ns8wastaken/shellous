use crate::renderer::animation::easing::Easing;

pub type AnimSlot = u32;

pub struct AnimSpec {
    value: f32,
    duration: f32,
    easing: Easing,
}

impl AnimSpec {
    pub fn new(value: f32) -> Self {
        Self {
            value,
            duration: 0.2,
            easing: Easing::Linear,
        }
    }

    pub fn with_duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    pub fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }
}

struct AnimEntry {
    value: f32,
    target: f32,
    start_value: f32,
    start_time: f32,
    duration: f32,
    easing: Easing,
    in_active: bool,
}

impl AnimEntry {
    fn new(spec: AnimSpec) -> Self {
        Self {
            value: spec.value,
            target: spec.value,
            start_value: spec.value,
            start_time: 0.0,
            duration: spec.duration,
            easing: spec.easing,
            in_active: false,
        }
    }

    fn value_at(&self, now: f32) -> f32 {
        let elapsed = now - self.start_time;
        if elapsed >= self.duration {
            return self.target;
        }
        let t = self.easing.apply(elapsed / self.duration);
        self.start_value + (self.target - self.start_value) * t
    }
}

pub struct AnimationCache {
    entries: Vec<Option<AnimEntry>>,
    active: Vec<AnimSlot>,
    free: Vec<AnimSlot>,
}

impl AnimationCache {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            active: Vec::new(),
            free: Vec::new(),
        }
    }

    pub fn insert(&mut self, spec: AnimSpec) -> AnimSlot {
        let entry = AnimEntry::new(spec);
        if let Some(slot) = self.free.pop() {
            self.entries[slot as usize] = Some(entry);
            slot
        } else {
            let slot = self.entries.len() as AnimSlot;
            self.entries.push(Some(entry));
            slot
        }
    }

    pub fn remove(&mut self, slot: AnimSlot) {
        if self.entries[slot as usize].is_some() {
            self.entries[slot as usize] = None;
            self.free.push(slot);
        }
    }

    pub fn value(&self, slot: AnimSlot) -> f32 {
        self.entries[slot as usize]
            .as_ref()
            .map_or(0.0, |e| e.value)
    }

    pub fn target(&self, slot: AnimSlot) -> f32 {
        self.entries[slot as usize]
            .as_ref()
            .map_or(0.0, |e| e.target)
    }

    pub fn set_target(&mut self, slot: AnimSlot, target: f32, now: f32) {
        if let Some(entry) = &mut self.entries[slot as usize] {
            let from = entry.value_at(now);
            if from == target {
                return;
            }
            entry.start_value = from;
            entry.target = target;
            entry.start_time = now;
            if !entry.in_active {
                entry.in_active = true;
                self.active.push(slot);
            }
        }
    }

    pub fn tick(&mut self, now: f32) -> bool {
        let mut still = false;
        self.active.retain(|&slot| {
            match &mut self.entries[slot as usize] {
                Some(entry) => {
                    entry.value = entry.value_at(now);
                    let settled = entry.target == entry.start_value
                        || now - entry.start_time >= entry.duration;
                    if settled {
                        entry.in_active = false;
                        false
                    } else {
                        still = true;
                        true
                    }
                }
                None => false,
            }
        });
        still
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_read() {
        let mut cache = AnimationCache::new();
        let s = cache.insert(AnimSpec::new(42.0));
        assert_eq!(cache.value(s), 42.0);
    }

    #[test]
    fn set_target_ticks_and_settles() {
        let mut cache = AnimationCache::new();
        let s = cache.insert(AnimSpec::new(0.0).with_duration(1.0));
        cache.set_target(s, 100.0, 0.0);
        // mid-animation
        assert!(cache.tick(0.5));
        let mid = cache.value(s);
        assert!(mid > 0.0 && mid < 100.0);
        // settled
        assert!(!cache.tick(2.0));
        assert_eq!(cache.value(s), 100.0);
    }

    #[test]
    fn set_target_to_same_is_noop() {
        let mut cache = AnimationCache::new();
        let s = cache.insert(AnimSpec::new(50.0).with_duration(1.0));
        cache.set_target(s, 50.0, 0.0);
        assert!(!cache.tick(0.5));
        assert_eq!(cache.value(s), 50.0);
    }

    #[test]
    fn remove_reuses_slot() {
        let mut cache = AnimationCache::new();
        let a = cache.insert(AnimSpec::new(1.0));
        let b = cache.insert(AnimSpec::new(2.0));
        cache.remove(a);
        let c = cache.insert(AnimSpec::new(3.0));
        assert_eq!(c, a); // free list reuses slot 0
        assert_eq!(cache.value(b), 2.0);
        assert_eq!(cache.value(c), 3.0);
    }

    #[test]
    fn tick_idle_returns_false() {
        let mut cache = AnimationCache::new();
        let _ = cache.insert(AnimSpec::new(10.0));
        assert!(!cache.tick(99.0));
    }

    #[test]
    fn chasing_behavior() {
        let mut cache = AnimationCache::new();
        let dot = cache.insert(AnimSpec::new(10.0).with_duration(1.0));
        let panel = cache.insert(AnimSpec::new(0.0).with_duration(0.5));

        // Dot starts animating toward 30.0
        cache.set_target(dot, 30.0, 0.0);
        cache.tick(0.5);
        let dot_mid = cache.value(dot);

        // Panel chases dot's current value
        cache.set_target(panel, dot_mid, 0.5);
        cache.tick(0.75); // panel advances a bit
        assert_eq!(cache.target(panel), dot_mid);

        // Final settle
        cache.tick(2.0);
        assert!(!cache.tick(3.0));
        assert_eq!(cache.value(dot), 30.0);
        assert_eq!(cache.value(panel), dot_mid);
    }
}
