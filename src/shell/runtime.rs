use std::cell::Cell;
use std::os::unix::io::AsFd;
use std::sync::Arc;
use std::time::Instant;

use calloop::{
    channel::{self, Sender},
    generic::Generic,
    EventLoop, Interest, Mode, PostAction,
};

use crate::components::ui::Element;
use crate::renderer::Renderer;
use crate::shell::compositor::Compositor;
use crate::shell::egl::EglState;
use crate::shell::event::{ShellEvent, ShellModule};
use crate::shell::layer_surface::{LayerSurface, ShellAnchor, ShellLayer};
use crate::shell::managed_surface::ManagedSurface;
use crate::shell::state::ShellState;
use crate::shell::surface::{Surface, SurfaceKind};
use crate::shell::surface_id::SurfaceId;
use crate::shell::wayland::WaylandState;
use crate::shell::xdg_surface::XdgToplevelSurface;

// ==================== SURFACE SPEC POLYMORPHISM ====================

pub struct LayerSpec {
    pub namespace: String,
    pub anchor: ShellAnchor,
    pub width: i32,
    pub height: i32,
    pub exclusive_zone: i32,
    pub layer: ShellLayer,
    pub root: Option<Box<dyn Element>>,
}

pub struct ToplevelSpec {
    pub title: String,
    pub app_id: String,
    pub min_size: Option<(i32, i32)>,
    pub max_size: Option<(i32, i32)>,
    pub root: Option<Box<dyn Element>>,
}

pub enum SurfaceSpec {
    Layer(LayerSpec),
    Toplevel(ToplevelSpec),
}

// ==================== CALLOP LOOP DATA ====================

/// All mutable state that calloop source callbacks can access.
pub struct LoopData {
    pub wayland: WaylandState,
    pub state: ShellState,
    pub egl: Arc<EglState>,
    pub anim_start: Instant,
    pub event_tx: Sender<ShellEvent>,
}

impl LoopData {
    pub fn process_wayland(&mut self) {
        let _ = self.wayland.conn.flush();
        loop {
            let count = self.wayland.event_queue.dispatch_pending(&mut self.state);
            if count.unwrap_or(0) == 0 {
                break;
            }
        }
        if let Some(guard) = self.wayland.event_queue.prepare_read() {
            if guard.read().is_ok() {
                loop {
                    let count = self.wayland.event_queue.dispatch_pending(&mut self.state);
                    if count.unwrap_or(0) == 0 {
                        break;
                    }
                }
            }
        }
    }

    pub fn handle_event(&mut self, event: ShellEvent) {
        self.state.update_surfaces(&event);
    }

    fn render_frame(&mut self) {
        if !self.state.any_dirty() {
            return;
        }

        let absolute_time = self.anim_start.elapsed().as_secs_f32();
        let still_moving = self.state.tick_animations(absolute_time);

        if still_moving {
            let qh = self.wayland.qh();
            for entry in &self.state.surfaces {
                if entry.animating.get() && entry.renderer.is_some() {
                    entry.request_frame(qh);
                }
            }
        }

        self.state.compute_layouts();
        self.state.render();

        for entry in &mut self.state.surfaces {
            entry.dirty.set(false);
        }

        let _ = self.wayland.conn.flush();
    }
}

// ==================== SHELL ====================

pub struct Shell {
    wayland: WaylandState,
    state: ShellState,
    egl: Arc<EglState>,
    event_tx: Sender<ShellEvent>,
    event_rx: channel::Channel<ShellEvent>,
}

impl Shell {
    pub fn new(compositor: Arc<dyn Compositor>) -> Self {
        let (event_tx, event_rx) = channel::channel::<ShellEvent>();

        let wayland = WaylandState::new();
        let state = ShellState::new(Arc::clone(&compositor));
        let egl = EglState::new(&wayland.conn);

        Self {
            wayland,
            state,
            egl,
            event_tx,
            event_rx,
        }
    }

    pub fn compositor(&self) -> &Arc<dyn Compositor> {
        &self.state.compositor
    }

    pub fn mount(&mut self, config: SurfaceSpec) -> SurfaceId {
        let id = self.state.next_id;
        self.state.next_id += 1;

        let (kind, root): (SurfaceKind, Option<Box<dyn Element>>) = match config {
            SurfaceSpec::Layer(spec) => {
                let layer_surface = LayerSurface::new(
                    &self.wayland,
                    id,
                    &spec.namespace,
                    spec.layer.to_wayland(),
                    spec.anchor.to_wayland(),
                    spec.width as u32,
                    spec.height as u32,
                    spec.exclusive_zone,
                );
                layer_surface.wl_surface.commit();
                (SurfaceKind::Layer(layer_surface), spec.root)
            }
            SurfaceSpec::Toplevel(spec) => {
                let xdg_surface = XdgToplevelSurface::new(
                    &self.wayland,
                    id,
                    &spec.title,
                    &spec.app_id,
                );
                if let Some((w, h)) = spec.min_size {
                    xdg_surface.xdg_toplevel.set_min_size(w, h);
                }
                if let Some((w, h)) = spec.max_size {
                    xdg_surface.xdg_toplevel.set_max_size(w, h);
                }
                xdg_surface.wl_surface.commit();
                (SurfaceKind::Toplevel(xdg_surface), spec.root)
            }
        };

        self.state.register(ManagedSurface {
            id,
            root,
            kind,
            renderer: None,
            frame_pending: Cell::new(false),
            dirty: Cell::new(true),
            animating: Cell::new(false),
            layout: None,
        });

        let surface_state_arc = {
            let surface = self.state.find_surface(id).unwrap();
            Arc::clone(surface.kind.surface_state())
        };
        self.wayland
            .wait_for_configure(&mut self.state, &surface_state_arc);

        let dims = {
            let surface = self.state.find_surface(id).unwrap();
            surface.kind.dimensions()
        };
        let surface = self.state.find_surface_mut(id).unwrap();
        surface.renderer = Some(Renderer::new(
            self.egl.clone(),
            surface.kind.wl_surface(),
            dims.0,
            dims.1,
        ));

        eprintln!("[shell] surface {id} ready ({}x{})", dims.0, dims.1);
        id
    }

    pub fn run(self, modules: Vec<Box<dyn ShellModule>>) {
        let mut event_loop = EventLoop::<LoopData>::try_new()
            .expect("initialize calloop event loop");

        let handle = event_loop.handle();

        // 1. Core Channel Listener
        handle
            .insert_source(
                self.event_rx,
                |event, _metadata, data: &mut LoopData| {
                    match event {
                        channel::Event::Msg(evt) => data.handle_event(evt),
                        channel::Event::Closed => {}
                    }
                    data.render_frame();
                },
            )
            .expect("insert_source event_rx");

        // 2. Wayland Descriptor Listener
        let wayland_fd = self
            .wayland
            .conn
            .as_fd()
            .try_clone_to_owned()
            .expect("clone wayland fd");
        handle
            .insert_source(
                Generic::new(wayland_fd, Interest::READ, Mode::Level),
                |_readiness, _file, data: &mut LoopData| {
                    data.process_wayland();
                    data.render_frame();
                    Ok(PostAction::Continue)
                },
            )
            .expect("insert_source wayland");

        // 3. Register pluggable background modules
        for module in &modules {
            module.register(&handle, self.event_tx.clone());
        }

        let mut data = LoopData {
            wayland: self.wayland,
            state: self.state,
            egl: self.egl,
            anim_start: Instant::now(),
            event_tx: self.event_tx,
        };

        for module in &modules {
            if let Some(evt) = module.initial_event() {
                data.handle_event(evt); // same update_surfaces() path as any later event
            }
        }

        data.render_frame();

        event_loop
            .run(None::<std::time::Duration>, &mut data, |_data| {})
            .expect("event_loop run");
    }
}
