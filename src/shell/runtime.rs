use std::cell::Cell;
use std::sync::Arc;

use crate::canvas::Canvas;
use crate::renderer::Renderer;
use crate::services::workspace::WorkspaceService;
use crate::shell::compositor::Compositor;
use crate::shell::egl::EglState;
use crate::shell::layer_surface::{LayerSurface, ShellAnchor, ShellLayer};
use crate::shell::managed_surface::ManagedSurface;
use crate::shell::state::ShellState;
use crate::shell::surface::{Surface, SurfaceKind};
use crate::shell::surface_id::SurfaceId;
use crate::shell::wayland::WaylandState;
use crate::shell::xdg_surface::XdgToplevelSurface;
use crate::ui::Element;

// ==================== SURFACE SPEC POLYMORPHISM ====================

/// Configuration for a zwlr-layer-shell panel surface.
pub struct LayerSpec {
    pub namespace: String,
    pub anchor: ShellAnchor,
    pub width: i32,
    pub height: i32,
    pub exclusive_zone: i32,
    pub layer: ShellLayer,
    pub elements: Vec<Box<dyn Element>>,
}

/// Configuration for an xdg-shell toplevel surface.
pub struct ToplevelSpec {
    pub title: String,
    pub app_id: String,
    pub min_size: Option<(i32, i32)>,
    pub max_size: Option<(i32, i32)>,
    pub elements: Vec<Box<dyn Element>>,
}

/// Polymorphic surface spec. `Layer` is the existing bar-panel kind;
/// `Toplevel` is the new xdg-shell window kind. New variants slot in
/// here as their scaffolding lands.
pub enum SurfaceSpec {
    Layer(LayerSpec),
    /// Scaffolding for future toplevel components. No current consumer
    /// in this codebase (bar uses `Layer`); silenced until the first
    /// toplevel mounts.
    #[allow(dead_code)]
    Toplevel(ToplevelSpec),
}

// ==================== SHELL ====================

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

        // Construct the right Wayland object for the configured kind,
        // set its params, commit once, and hand back (kind, elements).
        let (kind, elements): (SurfaceKind, Vec<Box<dyn Element>>) = match config {
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
                (SurfaceKind::Layer(layer_surface), spec.elements)
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
                (SurfaceKind::Toplevel(xdg_surface), spec.elements)
            }
        };

        self.state.register(ManagedSurface {
            id,
            elements,
            kind,
            renderer: None,
            frame_pending: Cell::new(false),
            dirty: Cell::new(true),
        });

        // 1. wait for the compositor's first Configure (configure carries
        //    the initial size on the surface_kind via its dispatch).
        let surface_state_arc = {
            let surface = self.state.find_surface(id).unwrap();
            Arc::clone(surface.kind.surface_state())
        };
        self.wayland
            .wait_for_configure(&mut self.state, &surface_state_arc);

        // 2. create the EGL-backed renderer now that dimensions are known.
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

    pub fn run(&mut self) {
        loop {
            self.wayland.dispatch_pending(&mut self.state);

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

            self.wayland.blocking_dispatch(&mut self.state);
        }
    }
}
