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
