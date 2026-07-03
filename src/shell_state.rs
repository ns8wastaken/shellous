use std::sync::{Arc, Mutex};

use crate::bar::BarState;
use crate::compositor::Compositor;

/// Global shell state — shared across all surfaces.
pub struct ShellState {
    /// Workspace list + active workspace (filled by the compositor backend).
    pub bar: Arc<Mutex<BarState>>,
    /// Back-end for workspace switching (Hyprland currently).
    pub compositor: Arc<dyn Compositor>,
    /// Current pointer position relative to whichever surface has focus.
    /// Only one surface can have the pointer at a time, so a single value is fine.
    pub pointer_pos: Option<(f64, f64)>,
    /// Height of the surface the pointer is currently on (for hit-testing in button clicks).
    pub pointer_surface_height: f32,
}
