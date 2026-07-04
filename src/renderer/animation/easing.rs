// ==================== EASING ====================

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    Linear,
    EaseOutQuad,
}

impl Easing {
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            Self::Linear => t,
            Self::EaseOutQuad => 1.0 - (1.0 - t) * (1.0 - t),
        }
    }
}
