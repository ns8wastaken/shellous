use wayland_client::{
    Connection, Dispatch, Proxy, QueueHandle,
    globals::{registry_queue_init, GlobalListContents},
    protocol::{
        wl_compositor::WlCompositor,
        wl_pointer::{ButtonState, Event as PointerEvent, WlPointer},
        wl_registry,
        wl_seat::{Capability, Event as SeatEvent, WlSeat},
        wl_surface::WlSurface,
    },
};

use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::{Layer, ZwlrLayerShellV1},
    zwlr_layer_surface_v1::{Anchor, Event as LayerEvent, ZwlrLayerSurfaceV1},
};

use khronos_egl as egl;
use wayland_egl::WlEglSurface;

use serde::Deserialize;

use std::ffi::CString;
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::UnixStream;
use std::ptr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use gl::types::*;

// ==================== SHADER SOURCES ====================

const VERT_SRC: &str = r#"
#version 330
layout(location = 0) in vec2 pos;
out vec2 uv;

void main() {
    uv = pos * 0.5 + 0.5;
    gl_Position = vec4(pos, 0.0, 1.0);
}
"#;

const FRAG_SRC: &str = r#"
#version 330

in vec2 uv;
out vec4 color;

uniform vec2 resolution;
uniform int ws_count;
uniform int active_slot;
uniform int hover_slot;

float roundedBox(vec2 p, vec2 b, float r) {
    vec2 d = abs(p) - b + r;
    return length(max(d, 0.0)) - r;
}

void main() {
    vec2 p = uv * resolution;

    vec2 bar_center = vec2(resolution.x * 0.5, resolution.y * 0.5);
    vec2 bar_half = vec2(resolution.x * 0.5 - 4.0, resolution.y * 0.5 - 4.0);

    float bar = roundedBox(p - bar_center, bar_half, 10.0);
    float bar_a = smoothstep(1.5, 0.0, bar);

    vec3 col = vec3(0.10, 0.11, 0.13);

    for (int i = 0; i < ws_count; i++) {
        vec2 c = p - vec2(24.0 + float(i) * 32.0, resolution.y * 0.5);
        float d = roundedBox(c, vec2(11.0, 9.0), 4.0);
        float a = smoothstep(1.0, 0.0, d);

        vec3 btn_col;
        if (i == active_slot) {
            btn_col = vec3(0.36, 0.56, 1.0);
        } else if (i == hover_slot) {
            btn_col = vec3(0.50, 0.51, 0.56);
        } else {
            btn_col = vec3(0.28, 0.29, 0.32);
        }

        col = mix(col, btn_col, a);
    }

    color = vec4(col, bar_a);
}
"#;

// ==================== HYPRLAND IPC ====================

#[derive(Debug, Clone, Deserialize)]
struct Workspace {
    id: i32,
    #[allow(dead_code)]
    name: String,
}

#[derive(Debug, Deserialize)]
struct ActiveWorkspace {
    id: i32,
}

fn hypr_sockets() -> (String, String) {
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

fn get_workspaces(cmd_socket: &str) -> std::io::Result<Vec<Workspace>> {
    let raw = hypr_command(cmd_socket, "j/workspaces")?;
    let mut list: Vec<Workspace> = serde_json::from_str(&raw)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    list.sort_by_key(|w| w.id);
    Ok(list)
}

fn get_active_workspace(cmd_socket: &str) -> std::io::Result<i32> {
    let raw = hypr_command(cmd_socket, "j/activeworkspace")?;
    let active: ActiveWorkspace = serde_json::from_str(&raw)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(active.id)
}


fn switch_workspace(cmd_socket: &str, id: i32) {
    let cmd_socket = cmd_socket.to_string();
    let cmd = format!("dispatch hl.dsp.focus({{ workspace = {id} }})");
    thread::spawn(move || match hypr_command(&cmd_socket, &cmd) {
        Ok(resp) => eprintln!("[bar] {cmd} -> {:?}", resp.trim()),
        Err(e) => eprintln!("[bar] {cmd} FAILED: {e}"),
    });
}

// ==================== SHARED BAR STATE ====================

struct BarState {
    workspaces: Vec<Workspace>,
    active_id: i32,
}

fn refresh_bar_state(cmd_socket: &str, shared: &Arc<Mutex<BarState>>) {
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
fn spawn_event_listener(cmd_socket: String, evt_socket: String, shared: Arc<Mutex<BarState>>) {
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

// ==================== OPENGL HELPERS ====================

unsafe fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    unsafe {
        let shader = gl::CreateShader(ty);
        let c_str = CString::new(src).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        let mut success: GLint = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut len: GLint = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = vec![0u8; len as usize];
            gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            panic!("shader compile error: {}", String::from_utf8_lossy(&buf));
        }

        shader
    }
}

unsafe fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);

        let mut success: GLint = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        if success == 0 {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = vec![0u8; len as usize];
            gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            panic!("program link error: {}", String::from_utf8_lossy(&buf));
        }

        program
    }
}

// ==================== WORKSPACE BUTTON LAYOUT ====================
// Mirrors the shader's math exactly -- if you change one, change the other.

struct ButtonRect {
    cx: f32,
    cy: f32,
    hw: f32,
    hh: f32,
}

fn button_layout(count: usize, height: f32) -> Vec<ButtonRect> {
    (0..count)
        .map(|i| ButtonRect {
            cx: 24.0 + i as f32 * 32.0,
            cy: height * 0.5,
            hw: 11.0,
            hh: 9.0,
        })
        .collect()
}

fn hit_test(buttons: &[ButtonRect], x: f32, y: f32) -> Option<usize> {
    buttons.iter().position(|b| {
        x >= b.cx - b.hw
        && x <= b.cx + b.hw
        && y >= b.cy - b.hh
        && y <= b.cy + b.hh
    })
}

// ==================== APP STATE / DISPATCH ====================

struct AppState {
    configured: bool,
    width: i32,
    height: i32,
    pointer_pos: Option<(f64, f64)>,
    bar: Arc<Mutex<BarState>>,
    cmd_socket: String,
}

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for AppState {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlCompositor, ()> for AppState {
    fn event(
        _: &mut Self,
        _: &WlCompositor,
        _: wayland_client::protocol::wl_compositor::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlSurface, ()> for AppState {
    fn event(
        _: &mut Self,
        _: &WlSurface,
        _: wayland_client::protocol::wl_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrLayerShellV1, ()> for AppState {
    fn event(
        _: &mut Self,
        _: &ZwlrLayerShellV1,
        _: wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrLayerSurfaceV1, ()> for AppState {
    fn event(
        state: &mut Self,
        proxy: &ZwlrLayerSurfaceV1,
        event: LayerEvent,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let LayerEvent::Configure { serial, width, height } = event {
            proxy.ack_configure(serial);
            if width > 0 {
                state.width = width as i32;
            }
            if height > 0 {
                state.height = height as i32;
            }
            state.configured = true;
        }
    }
}

impl Dispatch<WlSeat, ()> for AppState {
    fn event(
        _: &mut Self,
        seat: &WlSeat,
        event: SeatEvent,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let SeatEvent::Capabilities { capabilities } = event {
            if let wayland_client::WEnum::Value(caps) = capabilities {
                if caps.contains(Capability::Pointer) {
                    seat.get_pointer(qh, ());
                }
            }
        }
    }
}

impl Dispatch<WlPointer, ()> for AppState {
    fn event(
        state: &mut Self,
        _: &WlPointer,
        event: PointerEvent,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            PointerEvent::Enter {
                surface_x,
                surface_y,
                ..
            } => {
                eprintln!("[bar] pointer enter at ({surface_x:.1}, {surface_y:.1})");
                state.pointer_pos = Some((surface_x, surface_y));
            }
            PointerEvent::Motion {
                surface_x,
                surface_y,
                ..
            } => {
                state.pointer_pos = Some((surface_x, surface_y));
            }
            PointerEvent::Leave { .. } => {
                eprintln!("[bar] pointer leave");
                state.pointer_pos = None;
            }
            PointerEvent::Button {
                button,
                state: btn_state,
                ..
            } => {
                const BTN_LEFT: u32 = 0x110;
                let is_press = matches!(
                    btn_state,
                    wayland_client::WEnum::Value(ButtonState::Pressed)
                );
                eprintln!(
                    "[bar] button event: button=0x{button:x} press={is_press} pointer_pos={:?}",
                    state.pointer_pos
                );
                if button == BTN_LEFT && is_press {
                    if let Some((px, py)) = state.pointer_pos {
                        let bar = state.bar.lock().unwrap();
                        let buttons = button_layout(bar.workspaces.len(), state.height as f32);
                        eprintln!(
                            "[bar] hit-testing ({px:.1}, {py:.1}) against {} buttons (ws={:?})",
                            buttons.len(),
                            bar.workspaces.iter().map(|w| w.id).collect::<Vec<_>>()
                        );
                        // Wayland pointer coords are top-left origin, y-down.
                        // The shader's `p` space is also top-left/y-down here
                        // since we sample uv against the same resolution
                        // without flipping -- if buttons feel offset on your
                        // compositor, flip py to (height - py).
                        match hit_test(&buttons, px as f32, py as f32) {
                            Some(idx) => {
                                let id = bar.workspaces[idx].id;
                                drop(bar);
                                eprintln!("[bar] hit button {idx} -> workspace {id}");
                                switch_workspace(&state.cmd_socket, id);
                            }
                            None => eprintln!("[bar] click missed all buttons"),
                        }
                    } else {
                        eprintln!("[bar] click but no pointer_pos recorded");
                    }
                }
            }
            _ => {}
        }
    }
}

// ==================== MAIN ====================

fn main() {
    let (cmd_socket, evt_socket) = hypr_sockets();

    let bar_state = Arc::new(Mutex::new(BarState {
        workspaces: Vec::new(),
        active_id: -1,
    }));
    refresh_bar_state(&cmd_socket, &bar_state);
    spawn_event_listener(cmd_socket.clone(), evt_socket, bar_state.clone());

    let conn = Connection::connect_to_env().unwrap();
    let (globals, mut event_queue) = registry_queue_init::<AppState>(&conn).unwrap();
    let qh = event_queue.handle();

    let mut state = AppState {
        configured: false,
        width: 1920,
        height: 34,
        pointer_pos: None,
        bar: bar_state.clone(),
        cmd_socket: cmd_socket.clone(),
    };

    let compositor = globals
        .bind::<WlCompositor, _, _>(&qh, 1..=5, ())
        .expect("wl_compositor not available");
    let layer_shell = globals
        .bind::<ZwlrLayerShellV1, _, _>(&qh, 1..=4, ())
        .expect("zwlr_layer_shell_v1 not available");
    let seat = globals
        .bind::<WlSeat, _, _>(&qh, 1..=8, ())
        .expect("wl_seat not available");
    let _ = &seat; // capabilities event (handled in Dispatch) requests the pointer

    let surface = compositor.create_surface(&qh, ());

    let layer_surface =
        layer_shell.get_layer_surface(&surface, None, Layer::Top, "rust-bar".into(), &qh, ());

    layer_surface.set_anchor(Anchor::Top | Anchor::Left | Anchor::Right);
    layer_surface.set_size(0, 34);
    layer_surface.set_exclusive_zone(34);

    surface.commit();

    while !state.configured {
        event_queue.blocking_dispatch(&mut state).unwrap();
    }

    // ---------------- EGL CONTEXT ----------------

    let lib = unsafe { libloading::Library::new("libEGL.so.1") }
        .expect("unable to find libEGL.so.1");
    let egl = unsafe { egl::DynamicInstance::<egl::EGL1_4>::load_required_from(lib) }
        .expect("unable to load libEGL.so.1");

    let display_ptr = conn.display().id().as_ptr() as *mut std::ffi::c_void;
    let egl_display = unsafe { egl.get_display(display_ptr).expect("eglGetDisplay failed") };
    egl.initialize(egl_display).expect("eglInitialize failed");
    egl.bind_api(egl::OPENGL_API).expect("eglBindAPI failed");

    let config_attribs = [
        egl::SURFACE_TYPE,
        egl::WINDOW_BIT,
        egl::RENDERABLE_TYPE,
        egl::OPENGL_BIT,
        egl::RED_SIZE,
        8,
        egl::GREEN_SIZE,
        8,
        egl::BLUE_SIZE,
        8,
        egl::ALPHA_SIZE,
        8,
        egl::NONE,
    ];
    let egl_config = egl
        .choose_first_config(egl_display, &config_attribs)
        .expect("eglChooseConfig failed")
        .expect("no matching EGL config found");

    let context_attribs = [
        egl::CONTEXT_MAJOR_VERSION,
        3,
        egl::CONTEXT_MINOR_VERSION,
        3,
        egl::CONTEXT_OPENGL_PROFILE_MASK,
        egl::CONTEXT_OPENGL_CORE_PROFILE_BIT,
        egl::NONE,
    ];
    let egl_context = egl
        .create_context(egl_display, egl_config, None, &context_attribs)
        .expect("eglCreateContext failed");

    let wl_egl_surface = WlEglSurface::new(surface.id(), state.width, state.height)
        .expect("failed to create wl_egl_surface");

    let egl_surface = unsafe {
        egl.create_window_surface(
            egl_display,
            egl_config,
            wl_egl_surface.ptr() as egl::NativeWindowType,
            None,
        )
    }
    .expect("eglCreateWindowSurface failed");

    egl.make_current(
        egl_display,
        Some(egl_surface),
        Some(egl_surface),
        Some(egl_context),
    )
    .expect("eglMakeCurrent failed");

    // Pace the render loop to vblank instead of busy-looping.
    let _ = egl.swap_interval(egl_display, 1);

    gl::load_with(|s| egl.get_proc_address(s).unwrap() as *const _);

    unsafe {
        let vs = compile_shader(VERT_SRC, gl::VERTEX_SHADER);
        let fs = compile_shader(FRAG_SRC, gl::FRAGMENT_SHADER);
        let program = link_program(vs, fs);
        gl::DeleteShader(vs);
        gl::DeleteShader(fs);

        let vertices: [f32; 12] = [
            -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0,
        ];

        let mut vao: GLuint = 0;
        let mut vbo: GLuint = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
            vertices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );
        gl::VertexAttribPointer(
            0,
            2,
            gl::FLOAT,
            gl::FALSE,
            2 * std::mem::size_of::<f32>() as GLsizei,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        gl::UseProgram(program);
        let res_loc = gl::GetUniformLocation(program, b"resolution\0".as_ptr() as _);
        let count_loc = gl::GetUniformLocation(program, b"ws_count\0".as_ptr() as _);
        let active_loc = gl::GetUniformLocation(program, b"active_slot\0".as_ptr() as _);
        let hover_loc = gl::GetUniformLocation(program, b"hover_slot\0".as_ptr() as _);

        loop {
            event_queue.roundtrip(&mut state).unwrap();

            let (ws_count, active_slot) = {
                let bar = state.bar.lock().unwrap();
                let active_slot = bar
                    .workspaces
                    .iter()
                    .position(|w| w.id == bar.active_id)
                    .map(|i| i as i32)
                    .unwrap_or(-1);
                (bar.workspaces.len(), active_slot)
            };

            let hover_slot = state
                .pointer_pos
                .and_then(|(px, py)| {
                    let buttons = button_layout(ws_count, state.height as f32);
                    hit_test(&buttons, px as f32, py as f32)
                })
                .map(|i| i as i32)
                .unwrap_or(-1);

            gl::Viewport(0, 0, state.width, state.height);
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(program);
            gl::Uniform2f(res_loc, state.width as f32, state.height as f32);
            gl::Uniform1i(count_loc, ws_count.min(16) as i32);
            gl::Uniform1i(active_loc, active_slot);
            gl::Uniform1i(hover_loc, hover_slot);

            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);

            egl.swap_buffers(egl_display, egl_surface)
                .expect("eglSwapBuffers failed");
        }
    }
}
