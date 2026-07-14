use std::sync::{Arc, Mutex};

use calloop::channel::Sender;
use calloop::LoopHandle;

use crate::shell::compositor::{Compositor, CompositorEvent, SubscriptionId};
use crate::shell::event::{ShellEvent, ShellModule};
use crate::shell::runtime::LoopData;

// ==================== VALUE TYPES ====================

#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: i32,
    pub name: String,
}

#[derive(Clone)]
pub struct WorkspaceState {
    pub workspaces: Vec<Workspace>,
    pub active_id: i32,
}

// ==================== WORKSPACE HANDLE ====================

#[derive(Debug, Clone)]
pub struct WorkspaceSnapshot {
    pub workspaces: Vec<Workspace>,
    pub active_id: i32,
}

#[derive(Clone)]
pub struct WorkspaceHandle {
    state: Arc<Mutex<WorkspaceState>>,
}

impl WorkspaceHandle {
    pub fn snapshot(&self) -> WorkspaceSnapshot {
        let s = self.state.lock().unwrap();
        WorkspaceSnapshot {
            workspaces: s.workspaces.clone(),
            active_id: s.active_id,
        }
    }
}

// ==================== WORKSPACE SERVICE ====================

struct SubscriptionCleanup {
    compositor: Arc<dyn Compositor>,
    id: SubscriptionId,
}

impl Drop for SubscriptionCleanup {
    fn drop(&mut self) {
        let _ = self.compositor.unsubscribe(self.id);
    }
}

pub struct WorkspaceService {
    state: Arc<Mutex<WorkspaceState>>,
    compositor: Arc<dyn Compositor>,
    // We remove subscription_cleanup here because the subscription will happen
    // inside the `register` method when the loop boots.
}

impl WorkspaceService {
    /// Simply initializes the state wrapper without attaching a channel sender yet.
    pub fn new(compositor: Arc<dyn Compositor>) -> Self {
        let state = Arc::new(Mutex::new(WorkspaceState {
            workspaces: Vec::new(),
            active_id: -1,
        }));

        compositor.refresh_state(&state);

        Self {
            state,
            compositor,
        }
    }

    pub fn handle(&self) -> WorkspaceHandle {
        WorkspaceHandle {
            state: Arc::clone(&self.state),
        }
    }
}

// Implement the core trait so that it completely manages its own background IPC subscription lifecycle
impl ShellModule for WorkspaceService {
    fn register(
        &self,
        _handle: &LoopHandle<'_, LoopData>,
        tx: Sender<ShellEvent>,
    ) {
        let state_for_cb = Arc::clone(&self.state);

        // This subscription leaks into the background thread pool safely and sends updates via `tx`
        self.compositor.clone().subscribe_workspace_change(Arc::new(move |event| match event {
            CompositorEvent::WorkspaceChanged { workspaces, active_id } => {
                let snapshot = {
                    let mut s = state_for_cb.lock().unwrap();
                    s.workspaces = workspaces;
                    s.active_id = active_id;
                    WorkspaceSnapshot {
                        workspaces: s.workspaces.clone(),
                        active_id: s.active_id,
                    }
                };

                let _ = tx.send(ShellEvent::WorkspaceUpdated(snapshot));
            }
        }));
    }

    fn initial_event(&self) -> Option<ShellEvent> {
        let s = self.state.lock().unwrap();
        Some(ShellEvent::WorkspaceUpdated(WorkspaceSnapshot {
            workspaces: s.workspaces.clone(),
            active_id: s.active_id,
        }))
    }
}
