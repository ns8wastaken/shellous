use std::cell::Cell;
use std::sync::Arc;

use crate::canvas::Canvas;
use crate::renderer::Renderer;
use crate::services::workspace::WorkspaceService;
use crate::shell::compositor::Compositor;
use crate::shell::egl::EglState;
use crate::shell::layer_surface::{LayerSurface, ShellAnchor, ShellLayer, WaylandState};
use crate::shell::managed_surface::ManagedSurface;
use crate::shell::state::ShellState;
use crate::shell::surface_id::SurfaceId;
use crate::ui::Element;

pub struct SurfaceSpec {
    pub namespace: String,
    pub anchor: ShellAnchor,
    pub width: i32,
    pub height: i32,
    pub exclusive_zone: i32,
    pub layer: ShellLayer,
    pub elements: Vec<Box<dyn Element>>,
}

pub struct Shell {
    wayland: WaylandState,
    state: ShellState,
    egl: Arc<EglState>,
    workspace: Arc<WorkspaceService>,
}

impl Shell {
    pub fn new(compositor: Arc<dyn Compositor>) -> Self {
        let wayland = WaylandState::new();
        let state = ShellState::new(Arc::clone(&compositor));
        let egl = EglState::new(&wayland.conn);
        let workspace = Arc::new(WorkspaceService::new(Arc::clone(&compositor)));
        Self {
            wayland,
            state,
            egl,
            workspace,
        }
    }

    pub fn compositor(&self) -> &Arc<dyn Compositor> {
        &self.state.compositor
    }

    pub fn workspace(&self) -> &Arc<WorkspaceService> {
        &self.workspace
    }

    pub fn mount(&mut self, config: SurfaceSpec) -> SurfaceId {
        let id = self.state.next_id;
        self.state.next_id += 1;

        let wayland_layer = config.layer.to_wayland();
        let wayland_anchor = config.anchor.to_wayland();

        let (layer, wl_surface) =
            LayerSurface::new(&self.wayland, &config.namespace, id, wayland_layer);

        layer.layer_surface.set_anchor(wayland_anchor);
        layer
            .layer_surface
            .set_size(config.width as u32, config.height as u32);
        layer
            .layer_surface
            .set_exclusive_zone(config.exclusive_zone);
        wl_surface.commit();

        self.state.register(ManagedSurface {
            id,
            elements: config.elements,
            layer,
            wl_surface,
            renderer: None,
            frame_pending: Cell::new(false),
            dirty: Cell::new(true),
        });

        let surface_state = {
            let surface = self.state.find_surface(id).unwrap();
            surface.layer.surface_state.clone()
        };
        self.wayland
            .wait_for_configure(&mut self.state, &surface_state);

        let (w, h) = {
            let surface = self.state.find_surface(id).unwrap();
            surface.layer.dimensions()
        };

        let surface = self.state.find_surface_mut(id).unwrap();
        surface.renderer = Some(Renderer::new(self.egl.clone(), &surface.wl_surface, w, h));

        eprintln!("[shell] surface {id} ready ({w}x{h})");
        id
    }

    pub fn run(&mut self) {
        loop {
            // 1. Process any already-buffered events (non-blocking)
            self.wayland.dispatch_pending(&mut self.state);

            // 2. Render all dirty surfaces
            let qh = self.wayland.qh().clone();
            for entry in &self.state.surfaces {
                if !entry.dirty.get() || entry.renderer.is_none() {
                    continue;
                }

                entry.request_frame(&qh);

                let renderer = entry.renderer.as_ref().unwrap();
                renderer.make_current();
                let ctx = entry.render_context(&self.state);
                let canvas = Canvas::new(renderer.rect_program());
                renderer.render_frame(&ctx, || {
                    entry.draw(&canvas, &ctx);
                });

                entry.dirty.set(false);
            }

            // 3. Flush and block until the next Wayland event arrives
            self.wayland.blocking_dispatch(&mut self.state);
        }
    }
}
