/// Actions that a Panel can emit to the Shell.
/// Returned by `Panel::on_click` and dispatched by `ShellState::handle_click`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// No action — keep iterating panels.
    None,
    /// Switch to the given workspace id.
    SwitchWorkspace(i32),
}
