use std::sync::{Arc, Mutex};

use crate::shell::compositor::{Compositor, CompositorEvent, SubscriptionId};
use crate::workspace::{Workspace, WorkspaceState};

// ==================== WORKSPACE HANDLE ====================

/// Cheap-to-clone snapshot of workspace state. Returned by
/// `WorkspaceHandle::snapshot` for components that want an at-rest copy
/// without holding the handle's lock.
#[derive(Debug, Clone)]
pub struct WorkspaceSnapshot {
    pub workspaces: Vec<Workspace>,
    pub active_id: i32,
}

/// Cheap-to-clone handle giving components read access to the shared
/// workspace state without exposing the underlying `Mutex`.
#[derive(Clone)]
pub struct WorkspaceHandle {
    state: Arc<Mutex<WorkspaceState>>,
}

impl WorkspaceHandle {
    /// One lock, returns a clone of the data.
    pub fn snapshot(&self) -> WorkspaceSnapshot {
        let s = self.state.lock().unwrap();
        WorkspaceSnapshot {
            workspaces: s.workspaces.clone(),
            active_id: s.active_id,
        }
    }

    /// Read state inside a closure while the lock is held. Useful when
    /// the caller needs multiple fields in one consistent view.
    pub fn read<R>(&self, f: impl FnOnce(&WorkspaceState) -> R) -> R {
        let s = self.state.lock().unwrap();
        f(&s)
    }
}

// ==================== WORKSPACE SERVICE ====================

/// RAII guard that removes the subscription when `WorkspaceService` is
/// dropped. Ensures the callback Arc held inside the compositor's
/// subscription map is released instead of leaked for the shell's whole
/// lifetime.
struct SubscriptionCleanup {
    compositor: Arc<dyn Compositor>,
    id: SubscriptionId,
}

impl Drop for SubscriptionCleanup {
    fn drop(&mut self) {
        // Best-effort unsubscribe. The compositor's listener thread will
        // still see this work happen through the next callback iteration.
        let _ = self.compositor.unsubscribe(self.id);
    }
}

/// Shell-level service that maintains the latest workspace state from the
/// running compositor. Built once by `Shell::new`; components clone a
/// `WorkspaceHandle` via `handle()` to read workspace data.
///
/// Note: `subscription_cleanup` is declared LAST so it drops FIRST
/// (Rust's reverse-declaration drop order). Guaranteeing the listener is
/// unsubscribed before any callback could possibly fire against a
/// torn-down `state` — soundness isn't at risk because callbacks hold
/// their own `Arc<Mutex<WorkspaceState>>` clone, but the ordering is the
/// right invariant.
pub struct WorkspaceService {
    state: Arc<Mutex<WorkspaceState>>,
    /// RAII guard. Rust's `dead_code` linter doesn't count `Drop` as a
    /// field read, hence the explicit `allow`. Drop semantics is its only
    /// purpose — see `Drop for SubscriptionCleanup`.
    #[allow(dead_code)]
    subscription_cleanup: SubscriptionCleanup,
}

impl WorkspaceService {
    /// Seed state once from the compositor, then install a subscription
    /// that updates the state from each typed `CompositorEvent`.
    pub fn new(compositor: Arc<dyn Compositor>) -> Self {
        let state = Arc::new(Mutex::new(WorkspaceState {
            workspaces: Vec::new(),
            active_id: -1,
        }));

        // 1. Seed once via the default `refresh_state` helper.
        compositor.refresh_state(&state);

        // 2. Subscribe to typed events from here on. Clone compositor so
        //    there's one for the subscription call plus one for cleanup.
        let state_for_cb = Arc::clone(&state);
        let compositor_for_cleanup = Arc::clone(&compositor);
        let id = compositor.subscribe_workspace_change(Arc::new(move |event| match event {
            CompositorEvent::WorkspaceChanged { workspaces, active_id } => {
                let mut s = state_for_cb.lock().unwrap();
                s.workspaces = workspaces;
                s.active_id = active_id;
            }
        }));

        Self {
            state,
            subscription_cleanup: SubscriptionCleanup {
                compositor: compositor_for_cleanup,
                id,
            },
        }
    }

    /// Returns a cloneable handle a component can store.
    pub fn handle(&self) -> WorkspaceHandle {
        WorkspaceHandle {
            state: Arc::clone(&self.state),
        }
    }
}
