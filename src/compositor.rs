use std::sync::{Arc, Mutex};

use crate::bar::{BarState, Workspace};

/// Abstract interface for compositor workspace operations.
/// Implementations bridge to a specific compositor (e.g. Hyprland via IPC).
pub trait Compositor: Send + 'static {
    /// Fetch all workspaces from the compositor.
    fn fetch_workspaces(&self) -> Vec<Workspace>;

    /// Fetch the currently active workspace id.
    fn fetch_active_workspace(&self) -> i32;

    /// Switch to the given workspace id.
    fn switch_workspace(&self, id: i32);

    /// Start a background event listener that updates `bar` when workspace
    /// events occur. Implementations should spawn a thread that calls
    /// `refresh_bar` on itself whenever relevant events fire.
    fn spawn_event_listener(self: Arc<Self>, bar: Arc<Mutex<BarState>>);

    /// Convenience: fetch workspaces + active and write them into `bar`.
    fn refresh_bar(&self, bar: &Arc<Mutex<BarState>>) {
        let workspaces = self.fetch_workspaces();
        let active_id = self.fetch_active_workspace();
        eprintln!(
            "[bar] refreshed: {} workspaces, active={active_id}",
            workspaces.len()
        );
        let mut s = bar.lock().unwrap();
        s.workspaces = workspaces;
        s.active_id = active_id;
    }
}
