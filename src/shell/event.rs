use calloop::{LoopHandle, channel::Sender};

use super::runtime::LoopData;
use crate::services::{clock::ClockSnapshot, workspace::WorkspaceSnapshot};

// ==================== SHELL EVENT ====================

#[derive(Clone, Debug)]
pub enum ShellEvent {
    ClockUpdated(ClockSnapshot),
    WorkspaceUpdated(WorkspaceSnapshot),
    // TrayChanged(..),
    // add MprisChanged(..), Notification(..) here.
}

// ==================== MODULE TRAIT ====================

/// A pluggable source that registers calloop event sources.
/// Each module gets a copy of the event sender to push ShellEvents
/// from background threads into the main loop.
pub trait ShellModule: Send + 'static {
    fn register(&self, handle: &LoopHandle<'_, LoopData>, tx: Sender<ShellEvent>);

    /// Optional event representing current state at mount time.
    /// Fed directly into the element tree before the first render,
    /// so components never observe a "before any event" state.
    fn initial_event(&self) -> Option<ShellEvent> { None }
}
