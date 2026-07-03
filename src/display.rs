use std::sync::{Arc, Mutex};

use crate::bar::BarState;
use crate::compositor::Compositor;

/// Application state — compositor-agnostic data consumed by the renderer.
pub struct AppState {
    pub configured: bool,
    pub width: i32,
    pub height: i32,
    pub pointer_pos: Option<(f64, f64)>,
    pub bar: Arc<Mutex<BarState>>,
    pub compositor: Arc<dyn Compositor>,
}
