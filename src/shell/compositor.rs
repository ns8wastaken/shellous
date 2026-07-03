use std::sync::{Arc, Mutex};

use crate::components::bar::{BarState, Workspace};

pub trait Compositor: Send + Sync {
    fn get_workspaces(&self) -> Vec<Workspace>;
    fn get_active_workspace(&self) -> i32;
    fn switch_workspace(&self, id: i32);
    fn spawn_event_listener(self: Arc<Self>, bar: Arc<Mutex<BarState>>);

    fn refresh_bar(&self, bar: &Arc<Mutex<BarState>>) {
        let workspaces = self.get_workspaces();
        let active_id = self.get_active_workspace();
        eprintln!(
            "[bar] refreshed: {} workspaces, active={active_id}",
            workspaces.len()
        );
        let mut s = bar.lock().unwrap();
        s.workspaces = workspaces;
        s.active_id = active_id;
    }
}
