use crate::hyprland;

/// Shared workspace state, populated by Hyprland IPC and consumed by the renderer.
pub struct BarState {
    pub workspaces: Vec<hyprland::Workspace>,
    pub active_id: i32,
}

// ==================== WORKSPACE INDICATOR LAYOUT ====================
// Mirrors the shader's math exactly -- if you change one, change the other.
//
// Each workspace is a clickable element:
//   - Inactive: small circle (dot)
//   - Active:   elongated horizontal capsule
// Layout: left-aligned, 22px spacing, starting at x=20.
// All hit areas use a generous uniform hw=9.0 to cover both dots and the capsule.

pub struct ButtonRect {
    pub cx: f32,
    pub cy: f32,
    pub hw: f32,
    pub hh: f32,
}

/// Returns hit areas for `count` workspaces matching the shader layout.
pub fn button_layout(count: usize, height: f32) -> Vec<ButtonRect> {
    let hh = height * 0.5;
    (0..count)
        .map(|i| ButtonRect {
            cx: 20.0 + i as f32 * 22.0,
            cy: hh,
            // Generous hit area covering both dots (r=2.5) and capsules (half-length 5.5 + r=3.5)
            hw: 9.0,
            hh: 6.0,
        })
        .collect()
}

pub fn hit_test(buttons: &[ButtonRect], x: f32, y: f32) -> Option<usize> {
    buttons.iter().position(|b| {
        x >= b.cx - b.hw && x <= b.cx + b.hw && y >= b.cy - b.hh && y <= b.cy + b.hh
    })
}
