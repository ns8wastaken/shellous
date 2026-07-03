mod element;
mod state;

use std::sync::{Arc, Mutex};

use crate::shell::compositor::Compositor;
use crate::shell::runtime::Shell;

pub use element::BarElement;
pub use state::{BarState, Workspace};

pub fn install(shell: &mut Shell, compositor: Arc<dyn Compositor>, state: Arc<Mutex<BarState>>) {
    compositor.refresh_bar(&state);
    compositor.clone().spawn_event_listener(state.clone());
    shell.add_bar(0, 36 + 18, vec![Box::new(BarElement::default())]);
}
