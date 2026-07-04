use serde::Deserialize;

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
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

/// RAII guard that decrements `listener_count` when the listener thread
/// exits (panic or normal). Each spawned thread installs one at entry so
/// the next subscriber can detect a dead listener and respawn.
struct ListenerIncarnation {
    count: Arc<AtomicUsize>,
}
impl Drop for ListenerIncarnation {
    fn drop(&mut self) {
        self.count.fetch_sub(1, Ordering::AcqRel);
    }
}

/// Hyprland implementation of the `Compositor` trait. Holds the paths to
/// the command and event sockets, keyed subscription map, and a counter
/// tracking whether a listener thread is currently alive.
pub struct HyprlandCompositor {
    cmd_socket: String,
    evt_socket: String,
    subs: Mutex<HashMap<SubscriptionId, StateCallback>>,
    /// Number of listener threads currently alive. Mutated atomically.
    listener_count: Arc<AtomicUsize>,
    /// Monotonically increasing SubscriptionId source.
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
            listener_count: Arc::new(AtomicUsize::new(0)),
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
        // Install the increment/decrement guard for the lifetime of this
        // thread. If we panic, the guard still runs in Drop and decrements
        // the counter so the next subscriber can respawn.
        let _guard = ListenerIncarnation {
            count: Arc::clone(&self.listener_count),
        };

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
                            // Build the event ONCE; clone for each
                            // subscriber. `CompositorEvent` is `Clone`.
                            let event = CompositorEvent::WorkspaceChanged {
                                workspaces: self.workspaces(),
                                active_id: self.active_workspace(),
                            };

                            // Snapshot IDs under brief lock, then re-acquire
                            // the lock per callback to extract the `Arc`
                            // invocation target. Panics inside a callback
                            // (or unsubscribe-from-callback calls) happen
                            // outside the lock, so the `subs` mutex can
                            // never be poisoned by a subscriber.
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

        // Spawn the listener thread iff none is currently alive. The
        // Drop guard on the listener ensures the count returns to 0 if
        // the thread exits, so a future subscriber can respawn. Two
        // concurrent first-time subscribers both observe `prev == 0`
        // and 1 respectively — only one spawns.
        let prev = self.listener_count.fetch_add(1, Ordering::AcqRel);
        if prev == 0 {
            let me = Arc::clone(&self);
            thread::spawn(move || me.run_listener());
        }
        id
    }

    fn unsubscribe(&self, id: SubscriptionId) -> bool {
        self.subs.lock().unwrap().remove(&id).is_some()
    }
}
