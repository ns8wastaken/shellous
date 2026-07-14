use std::sync::{Arc, Mutex};
use crate::services::workspace::{Workspace, WorkspaceState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(pub u64);

#[derive(Clone)]
pub enum CompositorEvent {
    WorkspaceChanged {
        workspaces: Vec<Workspace>,
        active_id: i32,
    },
}

pub type StateCallback = Arc<dyn Fn(CompositorEvent) + Send + Sync>;

pub trait Compositor: Send + Sync {
    fn workspaces(&self) -> Vec<Workspace>;
    fn active_workspace(&self) -> i32;
    fn activate_workspace(&self, id: i32);
    fn subscribe_workspace_change(self: Arc<Self>, callback: StateCallback) -> SubscriptionId;
    fn unsubscribe(&self, id: SubscriptionId) -> bool;

    /// Helper utility to pull the foundational seed state
    fn refresh_state(&self, state: &Arc<Mutex<WorkspaceState>>) {
        let mut s = state.lock().unwrap();
        s.workspaces = self.workspaces();
        s.active_id = self.active_workspace();
    }
}
