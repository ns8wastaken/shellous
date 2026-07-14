use crate::services::workspace::WorkspaceSnapshot;

// ==================== SHELL EVENT ====================

#[derive(Clone, Debug)]
pub enum ShellEvent {
    WorkspaceUpdated(WorkspaceSnapshot),
    // TrayChanged,
    // add MprisChanged(..), Notification(..) here.
    // Each variant needs ~3 lines: enum def, match arm in LoopData::handle_event,
    // and a ShellModule impl.
}

// ==================== MODULE TRAIT ====================

/// A pluggable source that registers calloop event sources.
/// Each module gets a copy of the event sender to push ShellEvents
/// from background threads into the main loop.
pub trait ShellModule: Send + 'static {
    fn register(
        &self,
        handle: &calloop::LoopHandle<'_, super::runtime::LoopData>,
        tx: calloop::channel::Sender<ShellEvent>,
    );
}
