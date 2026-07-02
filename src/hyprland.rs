use serde::Deserialize;

use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::BarState;

// ==================== HYPRLAND IPC ====================

#[derive(Debug, Clone, Deserialize)]
pub struct Workspace {
    pub id: i32,
    #[allow(dead_code)]
    pub name: String,
}

#[derive(Debug, Deserialize)]
struct ActiveWorkspace {
    id: i32,
}

pub fn hypr_sockets() -> (String, String) {
    let sig = std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .expect("HYPRLAND_INSTANCE_SIGNATURE not set -- is this running under Hyprland?");
    let runtime = std::env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR not set");
    (
        format!("{runtime}/hypr/{sig}/.socket.sock"),
        format!("{runtime}/hypr/{sig}/.socket2.sock"),
    )
}

fn hypr_command(cmd_socket: &str, cmd: &str) -> std::io::Result<String> {
    let mut stream = UnixStream::connect(cmd_socket)?;
    stream.write_all(cmd.as_bytes())?;
    stream.shutdown(std::net::Shutdown::Write)?;
    let mut resp = String::new();
    stream.read_to_string(&mut resp)?;
    Ok(resp)
}

pub fn get_workspaces(cmd_socket: &str) -> std::io::Result<Vec<Workspace>> {
    let raw = hypr_command(cmd_socket, "j/workspaces")?;
    let mut list: Vec<Workspace> = serde_json::from_str(&raw)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    list.sort_by_key(|w| w.id);
    Ok(list)
}

pub fn get_active_workspace(cmd_socket: &str) -> std::io::Result<i32> {
    let raw = hypr_command(cmd_socket, "j/activeworkspace")?;
    let active: ActiveWorkspace = serde_json::from_str(&raw)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(active.id)
}

pub fn switch_workspace(cmd_socket: &str, id: i32) {
    let cmd_socket = cmd_socket.to_string();
    let cmd = format!("dispatch hl.dsp.focus({{ workspace = {id} }})");
    thread::spawn(move || match hypr_command(&cmd_socket, &cmd) {
        Ok(resp) => eprintln!("[bar] {cmd} -> {:?}", resp.trim()),
        Err(e) => eprintln!("[bar] {cmd} FAILED: {e}"),
    });
}

// ==================== SHARED BAR STATE ====================

pub fn refresh_bar_state(cmd_socket: &str, shared: &Arc<Mutex<BarState>>) {
    let workspaces = get_workspaces(cmd_socket).unwrap_or_else(|e| {
        eprintln!("[bar] get_workspaces failed: {e}");
        Vec::new()
    });
    let active_id = get_active_workspace(cmd_socket).unwrap_or_else(|e| {
        eprintln!("[bar] get_active_workspace failed: {e}");
        -1
    });
    eprintln!(
        "[bar] refreshed: {} workspaces, active={active_id}",
        workspaces.len()
    );
    let mut s = shared.lock().unwrap();
    s.workspaces = workspaces;
    s.active_id = active_id;
}

/// Listens on Hyprland's event socket (.socket2.sock) and refreshes the
/// shared workspace list/active id whenever something workspace-related
/// happens. Reconnects if Hyprland restarts or the socket hiccups.
pub fn spawn_event_listener(cmd_socket: String, evt_socket: String, shared: Arc<Mutex<BarState>>) {
    thread::spawn(move || loop {
        match UnixStream::connect(&evt_socket) {
            Ok(stream) => {
                let reader = BufReader::new(stream);
                for line in reader.lines() {
                    let Ok(line) = line else { break };
                    if line.starts_with("workspace")
                        || line.starts_with("createworkspace")
                        || line.starts_with("destroyworkspace")
                        || line.starts_with("moveworkspace")
                        || line.starts_with("focusedmon")
                    {
                        refresh_bar_state(&cmd_socket, &shared);
                    }
                }
            }
            Err(_) => {
                thread::sleep(Duration::from_secs(1));
            }
        }
    });
}
