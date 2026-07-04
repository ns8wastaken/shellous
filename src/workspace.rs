/// Compositor-agnostic workspace data, shared by any component that needs it.
#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: i32,
    pub name: String,
}

/// Shared workspace state. Derived `Clone` so handles can snapshot under
/// one lock, then release.
#[derive(Clone)]
pub struct WorkspaceState {
    pub workspaces: Vec<Workspace>,
    pub active_id: i32,
}
