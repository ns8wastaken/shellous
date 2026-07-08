// ==================== SIZE ====================

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    pub w: f32,
    pub h: f32,
}

impl Size {
    pub fn new(w: f32, h: f32) -> Self {
        Self { w, h }
    }
}

// ==================== RECT ====================

#[derive(Clone, Copy, Debug, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    pub fn from_size(size: Size) -> Self {
        Self { x: 0.0, y: 0.0, w: size.w, h: size.h }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px <= self.x + self.w && py >= self.y && py <= self.y + self.h
    }

    pub fn size(&self) -> Size {
        Size { w: self.w, h: self.h }
    }

    pub fn inset(self, l: f32, t: f32, r: f32, b: f32) -> Self {
        Self {
            x: self.x + l,
            y: self.y + t,
            w: (self.w - l - r).max(0.0),
            h: (self.h - t - b).max(0.0),
        }
    }
}

// ==================== ALIGNMENT HELPERS ====================

/// Returns (x, y) to place `child` at the left edge, vertically centered.
pub fn align_left_center(parent: Rect, child: Size) -> (f32, f32) {
    (parent.x, parent.y + (parent.h - child.h) * 0.5)
}

/// Returns (x, y) to place `child` at the right edge, vertically centered.
pub fn align_right_center(parent: Rect, child: Size) -> (f32, f32) {
    (parent.x + parent.w - child.w, parent.y + (parent.h - child.h) * 0.5)
}

/// Returns (x, y) to place `child` centered both axes.
pub fn align_center(parent: Rect, child: Size) -> (f32, f32) {
    (
        parent.x + (parent.w - child.w) * 0.5,
        parent.y + (parent.h - child.h) * 0.5,
    )
}

/// Returns (x, y) to place `child` at top-left.
pub fn align_top_left(parent: Rect, _child: Size) -> (f32, f32) {
    (parent.x, parent.y)
}

/// Returns (x, y) to place `child` at top-center.
pub fn align_top_center(parent: Rect, child: Size) -> (f32, f32) {
    (parent.x + (parent.w - child.w) * 0.5, parent.y)
}

/// Returns (x, y) to place `child` at bottom-left.
pub fn align_bottom_left(parent: Rect, child: Size) -> (f32, f32) {
    (parent.x, parent.y + parent.h - child.h)
}

// ==================== STACK LAYOUT ====================

/// Arrange `sizes` horizontally within `bounds` with `spacing` between items.
/// Returns the absolute `Rect` for each item, left-to-right.
pub fn stack_horizontal(bounds: Rect, sizes: &[Size], spacing: f32) -> Vec<Rect> {
    let mut cx = bounds.x;
    let mut rects = Vec::with_capacity(sizes.len());
    for s in sizes {
        rects.push(Rect { x: cx, y: bounds.y, w: s.w, h: s.h });
        cx += s.w + spacing;
    }
    rects
}

/// Arrange `sizes` vertically within `bounds` with `spacing` between items.
/// Returns the absolute `Rect` for each item, top-to-bottom.
pub fn stack_vertical(bounds: Rect, sizes: &[Size], spacing: f32) -> Vec<Rect> {
    let mut cy = bounds.y;
    let mut rects = Vec::with_capacity(sizes.len());
    for s in sizes {
        rects.push(Rect { x: bounds.x, y: cy, w: s.w, h: s.h });
        cy += s.h + spacing;
    }
    rects
}
