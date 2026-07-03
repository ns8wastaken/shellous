use wayland_client::{
    Connection, Dispatch, QueueHandle,
    globals::GlobalListContents,
    protocol::{
        wl_compositor::WlCompositor,
        wl_pointer::{ButtonState, Event as PointerEvent, WlPointer},
        wl_registry,
        wl_seat::{Capability, Event as SeatEvent, WlSeat},
        wl_surface::WlSurface,
    },
};

use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::ZwlrLayerShellV1,
    zwlr_layer_surface_v1::{Event as LayerEvent, ZwlrLayerSurfaceV1},
};

use crate::bar::{button_layout, hit_test};
use crate::display::AppState;

// ==================== WAYLAND DISPATCH ====================

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
                        match hit_test(&buttons, px as f32, py as f32) {
                            Some(idx) => {
                                let id = bar.workspaces[idx].id;
                                drop(bar);
                                eprintln!("[bar] hit button {idx} -> workspace {id}");
                                state.compositor.switch_workspace(id);
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
