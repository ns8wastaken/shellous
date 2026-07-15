use serde::Deserialize;

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::shell::compositor::{
    Compositor, CompositorEvent, StateCallback, SubscriptionId,
};
use crate::services::workspace::Workspace;

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
    subs: Mutex<HashMap<SubscriptionId, StateCallback>>,
    next_sub_id: AtomicU64,
}

impl HyprlandCompositor {
    pub fn new() -> Self {
        let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
            .expect("HYPRLAND_INSTANCE_SIGNATURE not set -- is this running under Hyprland?");
        let runtime = std::env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR not set");

        Self {
            cmd_socket: format!("{runtime}/hypr/{sig}/.socket.sock"),
            evt_socket: format!("{runtime}/hypr/{sig}/.socket2.sock"),
            subs: Mutex::new(HashMap::new()),
            next_sub_id: AtomicU64::new(1),
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

    fn run_listener(self: Arc<Self>) {
        let is_target_event = |line: &str| {
            [
                "workspace",
                "createworkspace",
                "destroyworkspace",
                "moveworkspace",
                "focusedmon",
            ]
            .iter()
            .any(|evt| line.starts_with(evt))
        };

        loop {
            match UnixStream::connect(&self.evt_socket) {
                Ok(stream) => {
                    let reader = BufReader::new(stream);
                    for line in reader.lines() {
                        let Ok(line) = line else { break };

                        if is_target_event(&line) {
                            let event = CompositorEvent::WorkspaceChanged {
                                workspaces: self.workspaces(),
                                active_id: self.active_workspace(),
                            };

                            let ids: Vec<SubscriptionId> = self
                                .subs
                                .lock()
                                .unwrap()
                                .keys()
                                .copied()
                                .collect();
                            for id in ids {
                                let cb = self.subs.lock().unwrap().get(&id).cloned();
                                if let Some(cb) = cb {
                                    cb(event.clone());
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    thread::sleep(Duration::from_secs(1));
                }
            }

            thread::sleep(Duration::from_millis(100));
        }
    }
}

impl Compositor for HyprlandCompositor {
    fn workspaces(&self) -> Vec<Workspace> {
        let raw = match self.hypr_command("j/workspaces") {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[compositor] workspaces failed: {e}");
                return Vec::new();
            }
        };
        let mut list: Vec<HyprWorkspace> = match serde_json::from_str(&raw) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[compositor] workspaces parse failed: {e}");
                return Vec::new();
            }
        };
        list.sort_by_key(|w| w.id);
        list.into_iter()
            .map(|w| Workspace { id: w.id, name: w.name })
            .collect()
    }

    fn active_workspace(&self) -> i32 {
        let raw = match self.hypr_command("j/activeworkspace") {
            Ok(r) => r,
            Err(e) => {
                eprintln!("[compositor] active_workspace failed: {e}");
                return -1;
            }
        };
        match serde_json::from_str::<ActiveWorkspace>(&raw) {
            Ok(a) => a.id,
            Err(e) => {
                eprintln!("[compositor] active_workspace parse failed: {e}");
                -1
            }
        }
    }

    fn activate_workspace(&self, id: i32) {
        let cmd = format!("dispatch hl.dsp.focus({{ workspace = {id} }})");
        match UnixStream::connect(&self.cmd_socket) {
            Ok(mut stream) => {
                let _ = stream.write_all(cmd.as_bytes());
                let _ = stream.shutdown(std::net::Shutdown::Write);
            }
            Err(e) => {
                eprintln!("[compositor] activate {id} FAILED: {e}");
            }
        }
    }

    fn subscribe_workspace_change(
        self: Arc<Self>,
        callback: StateCallback,
    ) -> SubscriptionId {
        let id = SubscriptionId(self.next_sub_id.fetch_add(1, Ordering::SeqCst));
        self.subs.lock().unwrap().insert(id, callback);

        let me = Arc::clone(&self);
        thread::spawn(move || me.run_listener());

        id
    }

    fn unsubscribe(&self, id: SubscriptionId) -> bool {
        self.subs.lock().unwrap().remove(&id).is_some()
    }
}
