use std::sync::{Arc, Mutex};

use crate::workspace::{Workspace, WorkspaceState};

// ==================== EVENT TYPE ====================

/// Typed event delivered to subscribers when compositor state changes.
/// `Clone` because the listener synthesises one event per tick and clones
/// it for each subscriber (we release the subscription lock during calls).
#[derive(Clone, Debug)]
pub enum CompositorEvent {
    /// Emitted whenever the set of workspaces or the active workspace
    /// changes. Backend implementations are responsible for emitting this
    /// only when something actually changed.
    WorkspaceChanged {
        workspaces: Vec<Workspace>,
        active_id: i32,
    },
}

// ==================== SUBSCRIPTION ====================

/// Unique identifier returned from `subscribe_workspace_change`. Pass it
/// back to `unsubscribe` to remove the callback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriptionId(pub u64);

/// Subscriber callback. We use `Arc<dyn Fn ...>` (not `Box<dyn Fn ...>`)
/// because `Arc` is `Clone` — the listener snapshots each callback's
/// `Arc` and invokes it outside the subscription lock to avoid mutex
/// poisoning on panic.
pub type StateCallback = Arc<dyn Fn(CompositorEvent) + Send + Sync>;

// ==================== TRAIT ====================

pub trait Compositor: Send + Sync {
    fn workspaces(&self) -> Vec<Workspace>;
    fn active_workspace(&self) -> i32;
    fn activate_workspace(&self, id: i32);

    /// Register a callback fired on every `CompositorEvent` of relevance.
    /// Returns a `SubscriptionId` that can later be passed to
    /// `unsubscribe`. Multi-subscriber; the backend manages however many
    /// listeners it needs.
    fn subscribe_workspace_change(
        self: Arc<Self>,
        callback: StateCallback,
    ) -> SubscriptionId;

    /// Remove a callback by ID. Returns `true` if an entry was removed.
    fn unsubscribe(&self, id: SubscriptionId) -> bool;

    /// Pull latest state from the compositor and write it into `state`.
    /// Used for initial hydration; subscribers keep state fresh from
    /// typed events afterward.
    fn refresh_state(&self, state: &Arc<Mutex<WorkspaceState>>) {
        let mut s = state.lock().unwrap();
        s.workspaces = self.workspaces();
        s.active_id = self.active_workspace();
    }
}
