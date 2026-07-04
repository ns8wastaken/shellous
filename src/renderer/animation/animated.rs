use crate::renderer::animation::easing::Easing;

// ==================== LERP ====================

pub trait Lerp: Copy {
    fn lerp(a: Self, b: Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }
}

// ==================== ANIMATED ====================

/// Time-based animated property.
///
/// `value(now)` is a pure function of `absolute_time` — no per-frame
/// mutation needed. Set the target with `set_target(target, now)` and
/// let the elapsed time drive the interpolation.
#[derive(Clone)]
pub struct Animated<T: Lerp + Copy> {
    current: T,
    target: T,
    start_value: T,
    start_time: f32,
    duration_secs: f32,
    easing: Easing,
}

impl<T: Lerp + Copy + PartialEq> Animated<T> {
    pub fn new(value: T) -> Self {
        Self {
            current: value,
            target: value,
            start_value: value,
            start_time: 0.0,
            duration_secs: 0.2,
            easing: Easing::EaseOutQuad,
        }
    }

    pub fn new_duration(value: T, duration_secs: f32, easing: Easing) -> Self {
        Self {
            current: value,
            target: value,
            start_value: value,
            start_time: 0.0,
            duration_secs,
            easing,
        }
    }

    pub fn set_target(&mut self, target: T, now: f32) {
        let from = self.value(now);
        if from == target {
            self.current = target;
            self.target = target;
            self.start_value = target;
            self.start_time = now - self.duration_secs;
            return;
        }
        self.current = from;
        self.start_value = from;
        self.target = target;
        self.start_time = now;
    }

    pub fn value(&self, now: f32) -> T {
        let elapsed = now - self.start_time;
        if elapsed >= self.duration_secs {
            return self.target;
        }
        let t = self.easing.apply(elapsed / self.duration_secs);
        T::lerp(self.start_value, self.target, t)
    }

    pub fn is_idle(&self, now: f32) -> bool {
        now - self.start_time >= self.duration_secs
    }
}
