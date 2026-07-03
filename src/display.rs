use std::sync::{Arc, Mutex};

use crate::bar::BarState;
use crate::compositor::Compositor;

/// Per-surface state — each layer surface gets its own instance.
/// Configured by the compositor's Configure event (not by the client's request).
pub struct SurfaceState {
    pub configured: bool,
    pub width: i32,
    pub height: i32,
}

impl SurfaceState {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            configured: false,
            width,
            height,
        }
    }
}

/// Global shell state — shared across all surfaces.
pub struct ShellState {
    /// Workspace list + active workspace (filled by the compositor backend).
    pub bar: Arc<Mutex<BarState>>,
    /// Back-end for workspace switching (Hyprland currently).
    pub compositor: Arc<dyn Compositor>,
    /// Current pointer position relative to whichever surface has focus.
    /// Only one surface can have the pointer at a time, so a single value is fine.
    pub pointer_pos: Option<(f64, f64)>,
}
