use std::sync::{Arc, Mutex};

use wayland_protocols_wlr::layer_shell::v1::client::{
    zwlr_layer_shell_v1::Layer, zwlr_layer_surface_v1::Anchor,
};

use crate::components::bar::BarState;
use crate::renderer::Renderer;
use crate::shell::compositor::Compositor;
use crate::shell::egl::EglState;
use crate::shell::layer_surface::{LayerSurface, WaylandState};
use crate::shell::managed_surface::ManagedSurface;
use crate::shell::state::ShellState;
use crate::shell::surface_id::SurfaceId;
use crate::ui::{Element, SurfaceModel, SurfaceRole};

pub struct SurfaceSpec {
    pub namespace: String,
    pub anchor: Anchor,
    pub width: i32,
    pub height: i32,
    pub exclusive_zone: i32,
    pub layer: Layer,
    pub role: SurfaceRole,
    pub elements: Vec<Box<dyn Element>>,
}

pub struct Shell {
    wayland: WaylandState,
    state: ShellState,
    egl: Arc<EglState>,
}

impl Shell {
    pub fn new(compositor: Arc<dyn Compositor>) -> Self {
        let wayland = WaylandState::new();
        let state = ShellState::new(compositor);
        let egl = EglState::new(&wayland.conn);
        Self {
            wayland,
            state,
            egl,
        }
    }

    pub fn compositor(&self) -> &Arc<dyn Compositor> {
        &self.state.compositor
    }

    pub fn bar_state(&self) -> &Arc<Mutex<BarState>> {
        &self.state.bar
    }

    pub fn mount(&mut self, config: SurfaceSpec) -> SurfaceId {
        let id = self.state.next_id;
        self.state.next_id += 1;

        let (layer, wl_surface) =
            LayerSurface::new(&self.wayland, &config.namespace, id, config.layer);

        layer.layer_surface.set_anchor(config.anchor);
        layer
            .layer_surface
            .set_size(config.width as u32, config.height as u32);
        layer
            .layer_surface
            .set_exclusive_zone(config.exclusive_zone);
        wl_surface.commit();

        self.state.register(ManagedSurface {
            id,
            layer,
            wl_surface,
            renderer: None,
            model: SurfaceModel::new(config.role, config.elements),
            frame_pending: false,
            dirty: true,
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
            self.wayland.dispatch(&mut self.state);
            let qh = self.wayland.qh().clone();

            let dirty_ids: Vec<SurfaceId> = self.state.surfaces.iter()
                .filter(|e| e.dirty && e.renderer.is_some())
                .map(|e| e.id)
                .collect();

            // Pass 1: queue frame callbacks BEFORE each surface's commit (mutable)
            for id in &dirty_ids {
                if let Some(entry) = self.state.find_surface_mut(*id) {
                    entry.request_frame(&qh);
                }
            }

            // Pass 2: render — swap triggers the commit that flushes the queued callback (immutable)
            for entry in &self.state.surfaces {
                if !dirty_ids.contains(&entry.id) {
                    continue;
                }
                if let Some(ref renderer) = entry.renderer {
                    renderer.make_current();
                    let ctx = entry.render_context(&self.state);
                    renderer.render_frame(&ctx, || {
                        entry.model.tree.draw(renderer.rect_program(), &ctx);
                    });
                }
            }

            // Pass 3: clear dirty now that it's been rendered (mutable)
            for id in dirty_ids {
                if let Some(entry) = self.state.find_surface_mut(id) {
                    entry.dirty = false;
                }
            }
        }
    }
}
