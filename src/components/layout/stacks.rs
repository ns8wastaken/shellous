use crate::components::rect::{Rect, Size};

// ==================== STACK LAYOUT ====================

pub fn stack_horizontal(bounds: Rect, sizes: &[Size], spacing: f32) -> Vec<Rect> {
    let mut cx = bounds.x;
    let mut rects = Vec::with_capacity(sizes.len());
    for s in sizes {
        rects.push(Rect { x: cx, y: bounds.y, w: s.w, h: s.h });
        cx += s.w + spacing;
    }
    rects
}

pub fn stack_vertical(bounds: Rect, sizes: &[Size], spacing: f32) -> Vec<Rect> {
    let mut cy = bounds.y;
    let mut rects = Vec::with_capacity(sizes.len());
    for s in sizes {
        rects.push(Rect { x: bounds.x, y: cy, w: s.w, h: s.h });
        cy += s.h + spacing;
    }
    rects
}
