/// Compositor-agnostic workspace data.
#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: i32,
    pub name: String,
}

/// Shared workspace state, consumed by the renderer and wayland dispatch.
pub struct BarState {
    pub workspaces: Vec<Workspace>,
    pub active_id: i32,
}


