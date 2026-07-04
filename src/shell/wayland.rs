use std::sync::{Arc, Mutex};

use wayland_client::{
    Connection, Dispatch, EventQueue, QueueHandle,
    globals::{GlobalListContents, registry_queue_init},
    protocol::{
        wl_callback::{self, WlCallback},
        wl_compositor::WlCompositor,
        wl_pointer::{ButtonState, Event as PointerEvent, WlPointer},
        wl_registry,
        wl_seat::{Capability, Event as SeatEvent, WlSeat},
        wl_surface::WlSurface,
    },
};
use wayland_protocols::xdg::shell::client::{
    xdg_surface::{Event as XdgSurfaceEvent, XdgSurface},
    xdg_toplevel::{Event as XdgToplevelEvent, XdgToplevel},
    xdg_wm_base::{Event as XdgWmBaseEvent, XdgWmBase},
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::ZwlrLayerShellV1,
    zwlr_layer_surface_v1::{Event as LayerEvent, ZwlrLayerSurfaceV1},
};

use crate::shell::state::ShellState;
use crate::shell::surface::SurfaceState;
use crate::shell::surface_id::SurfaceId;

// ==================== WAYLAND STATE ====================

/// Owns the Wayland connection handle, event queue, and protocol globals
/// the shell needs (wl_compositor, layer_shell, xdg_wm_base, seat).
/// Dispatch impls below send events into `ShellState`.
pub struct WaylandState {
    pub conn: Connection,
    pub event_queue: EventQueue<ShellState>,
    pub qh: QueueHandle<ShellState>,
    pub layer_shell: ZwlrLayerShellV1,
    pub xdg_wm_base: XdgWmBase,
    pub wl_compositor: WlCompositor,
    #[allow(dead_code)]
    seat: WlSeat,
}

impl WaylandState {
    pub fn new() -> Self {
        let conn = Connection::connect_to_env().unwrap();
        let (globals, event_queue) = registry_queue_init::<ShellState>(&conn).unwrap();
        let qh = event_queue.handle();

        let wl_compositor = globals
            .bind::<WlCompositor, _, _>(&qh, 1..=5, ())
            .expect("wl_compositor not available");
        let layer_shell = globals
            .bind::<ZwlrLayerShellV1, _, _>(&qh, 1..=4, ())
            .expect("zwlr_layer_shell_v1 not available");
        let xdg_wm_base = globals
            .bind::<XdgWmBase, _, _>(&qh, 1..=6, ())
            .expect("xdg_wm_base not available");
        let seat = globals
            .bind::<WlSeat, _, _>(&qh, 1..=8, ())
            .expect("wl_seat not available");

        Self {
            conn,
            event_queue,
            qh,
            layer_shell,
            xdg_wm_base,
            wl_compositor,
            seat,
        }
    }

    pub fn dispatch_pending(&mut self, state: &mut ShellState) {
        self.event_queue.dispatch_pending(state).unwrap();
    }

    pub fn blocking_dispatch(&mut self, state: &mut ShellState) {
        self.event_queue.blocking_dispatch(state).unwrap();
    }

    pub fn qh(&self) -> &QueueHandle<ShellState> {
        &self.qh
    }

    pub fn wait_for_configure(
        &mut self,
        state: &mut ShellState,
        surface_state: &Arc<Mutex<SurfaceState>>,
    ) {
        loop {
            if surface_state.lock().unwrap().configured {
                break;
            }
            self.event_queue.blocking_dispatch(state).unwrap();
        }
    }
}

// ==================== DISPATCH IMPLS ====================

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for ShellState {
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

impl Dispatch<WlCompositor, ()> for ShellState {
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

impl Dispatch<WlSurface, ()> for ShellState {
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

impl Dispatch<ZwlrLayerShellV1, ()> for ShellState {
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

impl Dispatch<ZwlrLayerSurfaceV1, Arc<Mutex<SurfaceState>>> for ShellState {
    fn event(
        _: &mut Self,
        proxy: &ZwlrLayerSurfaceV1,
        event: LayerEvent,
        data: &Arc<Mutex<SurfaceState>>,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let LayerEvent::Configure { serial, width, height } = event {
            proxy.ack_configure(serial);
            let mut ss = data.lock().unwrap();
            if width > 0 {
                ss.width = width as i32;
            }
            if height > 0 {
                ss.height = height as i32;
            }
            ss.configured = true;
        }
    }
}

impl Dispatch<WlSeat, ()> for ShellState {
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

impl Dispatch<WlPointer, ()> for ShellState {
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
                surface,
                ..
            } => {
                state.set_focus_by_surface(&surface);
                state.pointer_pos = Some((surface_x, surface_y));
            }
            PointerEvent::Motion {
                surface_x,
                surface_y,
                ..
            } => state.pointer_pos = Some((surface_x, surface_y)),
            PointerEvent::Leave { .. } => {
                state.focused_surface = None;
                state.pointer_pos = None;
            }
            PointerEvent::Button {
                button,
                state: btn_state,
                ..
            } => {
                const BTN_LEFT: u32 = 0x110;
                let is_press =
                    matches!(btn_state, wayland_client::WEnum::Value(ButtonState::Pressed));
                if button == BTN_LEFT && is_press {
                    state.handle_click();
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<WlCallback, SurfaceId> for ShellState {
    fn event(state: &mut Self, _: &WlCallback, event: wl_callback::Event, data: &SurfaceId, _: &Connection, _: &QueueHandle<Self>) {
        if let wl_callback::Event::Done { .. } = event {
            if let Some(surface) = state.find_surface_mut(*data) {
                surface.frame_pending.set(false);
                surface.dirty.set(true); // ready to draw next tick
            }
        }
    }
}

// ----- xdg-shell dispatches -----

impl Dispatch<XdgWmBase, ()> for ShellState {
    fn event(
        _: &mut Self,
        proxy: &XdgWmBase,
        event: XdgWmBaseEvent,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        // xdg-shell requires responding to every ping with a pong, or the
        // compositor treats the client as unresponsive.
        if let XdgWmBaseEvent::Ping { serial } = event {
            proxy.pong(serial);
        }
    }
}

impl Dispatch<XdgSurface, Arc<Mutex<SurfaceState>>> for ShellState {
    fn event(
        _: &mut Self,
        proxy: &XdgSurface,
        event: XdgSurfaceEvent,
        data: &Arc<Mutex<SurfaceState>>,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let XdgSurfaceEvent::Configure { serial } = event {
            proxy.ack_configure(serial);
            data.lock().unwrap().configured = true;
        }
    }
}

impl Dispatch<XdgToplevel, Arc<Mutex<SurfaceState>>> for ShellState {
    fn event(
        _: &mut Self,
        _: &XdgToplevel,
        event: XdgToplevelEvent,
        data: &Arc<Mutex<SurfaceState>>,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let XdgToplevelEvent::Configure { width, height, states: _ } = event {
            // Compositor may pass 0 to indicate "pick a size" — keep last non-zero.
            let mut ss = data.lock().unwrap();
            if width > 0 {
                ss.width = width;
            }
            if height > 0 {
                ss.height = height;
            }
        }
    }
}
