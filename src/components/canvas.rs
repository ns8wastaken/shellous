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

    pub fn place_center(self, child: Size) -> Self {
        Self {
            x: self.x + (self.w - child.w) * 0.5,
            y: self.y + (self.h - child.h) * 0.5,
            w: child.w,
            h: child.h,
        }
    }

    pub fn place_left_center(self, child: Size) -> Self {
        Self {
            x: self.x,
            y: self.y + (self.h - child.h) * 0.5,
            w: child.w,
            h: child.h,
        }
    }

    pub fn place_right_center(self, child: Size) -> Self {
        Self {
            x: self.x + self.w - child.w,
            y: self.y + (self.h - child.h) * 0.5,
            w: child.w,
            h: child.h,
        }
    }

    pub fn place_top_left(self, child: Size) -> Self {
        Self {
            x: self.x,
            y: self.y,
            w: child.w,
            h: child.h,
        }
    }

    pub fn place_top_center(self, child: Size) -> Self {
        Self {
            x: self.x + (self.w - child.w) * 0.5,
            y: self.y,
            w: child.w,
            h: child.h,
        }
    }

    pub fn place_bottom_left(self, child: Size) -> Self {
        Self {
            x: self.x,
            y: self.y + self.h - child.h,
            w: child.w,
            h: child.h,
        }
    }
}

// ==================== ALIGNMENT ====================

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Alignment {
    #[default]
    Fill,
    Center,
    TopCenter,
    Start,
    End,
}

impl Alignment {
    pub fn apply(self, parent: Rect, child: Size) -> Rect {
        match self {
            Alignment::Fill      => parent,
            Alignment::Center    => parent.place_center(child),
            Alignment::TopCenter => parent.place_top_center(child),
            Alignment::Start     => parent.place_left_center(child),
            Alignment::End       => parent.place_right_center(child),
        }
    }
}

// ==================== ALIGNMENT HELPERS ====================

pub fn align_center(parent: Rect, child: Size) -> Rect {
    parent.place_center(child)
}

pub fn align_left_center(parent: Rect, child: Size) -> Rect {
    parent.place_left_center(child)
}

pub fn align_right_center(parent: Rect, child: Size) -> Rect {
    parent.place_right_center(child)
}

pub fn align_top_left(parent: Rect, child: Size) -> Rect {
    parent.place_top_left(child)
}

pub fn align_top_center(parent: Rect, child: Size) -> Rect {
    parent.place_top_center(child)
}

pub fn align_bottom_left(parent: Rect, child: Size) -> Rect {
    parent.place_bottom_left(child)
}

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
