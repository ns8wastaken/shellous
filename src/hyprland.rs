use serde::Deserialize;

use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::components::bar::{BarState, Workspace};
use crate::shell::compositor::Compositor;

// ==================== INTERNAL JSON TYPES ====================

#[derive(Debug, Deserialize)]
struct HyprWorkspace {
    id: i32,
    name: String,
}

#[derive(Debug, Deserialize)]
struct ActiveWorkspace {
    id: i32,
}

// ==================== HYPRLAND COMPOSITOR ====================

pub struct HyprlandCompositor {
    cmd_socket: String,
    evt_socket: String,
}

impl HyprlandCompositor {
    pub fn new() -> Self {
        let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .expect("HYPRLAND_INSTANCE_SIGNATURE not set -- is this running under Hyprland?");
        let runtime = std::env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR not set");
        Self {
            cmd_socket: format!("{runtime}/hypr/{sig}/.socket.sock"),
            evt_socket: format!("{runtime}/hypr/{sig}/.socket2.sock"),
        }
    }

    fn hypr_command(&self, cmd: &str) -> std::io::Result<String> {
        let mut stream = UnixStream::connect(&self.cmd_socket)?;
        stream.write_all(cmd.as_bytes())?;
        stream.shutdown(std::net::Shutdown::Write)?;
        let mut resp = String::new();
        stream.read_to_string(&mut resp)?;
        Ok(resp)
    }
}

impl Compositor for HyprlandCompositor {
    fn get_workspaces(&self) -> Vec<Workspace> {
        let raw = match self.hypr_command("j/workspaces") {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[bar] get_workspaces failed: {e}");
                return Vec::new();
            }
        };
        let mut list: Vec<HyprWorkspace> = match serde_json::from_str(&raw) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[bar] get_workspaces parse failed: {e}");
                return Vec::new();
            }
        };
        list.sort_by_key(|w| w.id);
        list.into_iter()
            .map(|w| Workspace { id: w.id, name: w.name })
            .collect()
    }

    fn get_active_workspace(&self) -> i32 {
        let raw = match self.hypr_command("j/activeworkspace") {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[bar] get_active_workspace failed: {e}");
                return -1;
            }
        };
        match serde_json::from_str::<ActiveWorkspace>(&raw) {
            Ok(a) => a.id,
            Err(e) => {
                eprintln!("[bar] get_active_workspace parse failed: {e}");
                -1
            }
        }
    }

    fn switch_workspace(&self, id: i32) {
        let cmd = format!("dispatch hl.dsp.focus({{ workspace = {id} }})");
        match UnixStream::connect(&self.cmd_socket) {
            Ok(mut stream) => {
                let _ = stream.write_all(cmd.as_bytes());
                let _ = stream.shutdown(std::net::Shutdown::Write);
            }
            Err(e) => {
                eprintln!("[bar] switch to {id} FAILED: {e}");
            }
        }
    }

    fn spawn_event_listener(self: Arc<Self>, bar: Arc<Mutex<BarState>>) {
        // Clone the Arc of self so the thread can own a reference to the compositor
        let compositor = Arc::clone(&self);

        thread::spawn(move || loop {
            match UnixStream::connect(&compositor.evt_socket) {
                Ok(stream) => {
                    let reader = BufReader::new(stream);
                    for line in reader.lines() {
                        let Ok(line) = line else { break };

                        // Clean up the event matching logic
                        let is_target_event = [
                            "workspace",
                            "createworkspace",
                            "destroyworkspace",
                            "moveworkspace",
                            "focusedmon",
                        ]
                        .iter()
                        .any(|evt| line.starts_with(evt));

                        if is_target_event {
                            // Call refresh_bar directly using the cloned Arc
                            compositor.refresh_bar(&bar);
                        }
                    }
                }
                Err(_) => {
                    // Connection failed: sleep to prevent spinning
                    thread::sleep(Duration::from_secs(1));
                }
            }

            // Defensive sleep: prevents a tight loop if the connection
            // succeeds but immediately drops over and over.
            thread::sleep(Duration::from_millis(100));
        });
    }
}
