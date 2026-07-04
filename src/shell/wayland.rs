use std::sync::{Arc, Mutex};

use wayland_client::{
    Connection, Dispatch, QueueHandle, globals::GlobalListContents, protocol::{
        wl_callback::{self, WlCallback}, wl_compositor::WlCompositor, wl_pointer::{ButtonState, Event as PointerEvent, WlPointer}, wl_registry, wl_seat::{Capability, Event as SeatEvent, WlSeat}, wl_surface::WlSurface
    }
};
use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::ZwlrLayerShellV1,
    zwlr_layer_surface_v1::{Event as LayerEvent, ZwlrLayerSurfaceV1},
};

use crate::shell::{layer_surface::SurfaceState, surface_id::SurfaceId};
use crate::shell::state::ShellState;

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
    fn event(state: &mut Self, _: &WlCallback, event: wl_callback::Event, data: &SurfaceId, _: &Connection, qh: &QueueHandle<Self>) {
        if let wl_callback::Event::Done { .. } = event {
            if let Some(surface) = state.find_surface_mut(*data) {
                surface.frame_pending.set(false);
                surface.dirty.set(true); // ready to draw next tick
            }
        }
    }
}
