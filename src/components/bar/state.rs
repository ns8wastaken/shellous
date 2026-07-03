/// Compositor-agnostic workspace data.
#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: i32,
    pub name: String,
}

/// Shared workspace state, consumed by the renderer and wayland dispatch.
pub struct BarState {
    pub workspaces: Vec<Workspace>,
    pub active_id: i32,
}

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
