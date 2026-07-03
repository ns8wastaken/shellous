use std::sync::{Arc, Mutex};

use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::Anchor;

use crate::bar::BarState;
use crate::compositor::Compositor;
use crate::egl_state::EglState;
use crate::layer_surface::LayerSurface;
use crate::managed_surface::ManagedSurface;
use crate::renderer::panel::Panel;
use crate::renderer::Renderer;
use crate::layer_surface::WaylandState;
use crate::shell_state::ShellState;
use crate::surface_id::SurfaceId;

// ==================== SURFACE CONFIG ====================

/// Configuration for a new layer surface.
#[allow(dead_code)]
pub struct SurfaceConfig {
    /// Wayland namespace (used to identify the surface to the compositor).
    pub namespace: String,
    /// Anchor edges.
    pub anchor: Anchor,
    /// Requested size (0 means compositor decides).
    pub width: i32,
    pub height: i32,
    /// Exclusive zone (reserves space at the screen edge).
    pub exclusive_zone: i32,
    /// Panels to draw on this surface.
    pub panels: Vec<Box<dyn Panel>>,
}

// ==================== SHELL ====================

/// The main application — owns the Wayland connection, EGL state, and all
/// managed surfaces.  Call `add_surface` one or more times, then `run`.
pub struct Shell {
    wayland: WaylandState,
    state: ShellState,
    egl: Arc<EglState>,
}

impl Shell {
    /// Create a new shell.
    ///
    /// `compositor` is the back-end for workspace operations.
    /// `bar` is the shared workspace state (filled by the compositor listener).
    pub fn new(
        compositor: Arc<dyn Compositor>,
        bar: Arc<Mutex<BarState>>,
    ) -> Self {
        let wayland = WaylandState::new();
        let state = ShellState::new(bar, compositor);
        let egl = EglState::new(&wayland.conn);

        Self {
            wayland,
            state,
            egl,
        }
    }

    /// Add a new layer surface to the shell.
    ///
    /// This blocks until the compositor responds with a `Configure` event,
    /// so the returned `SurfaceId` has a fully-initialized renderer ready
    /// for the render loop.
    pub fn add_surface(&mut self, config: SurfaceConfig) -> SurfaceId {
        let id = self.state.next_id;
        self.state.next_id += 1;

        // ---- 1. Create Wayland layer surface ----
        let (layer, wl_surface) =
            LayerSurface::new(&self.wayland, &config.namespace, id);

        layer.layer_surface.set_anchor(config.anchor);
        layer.layer_surface.set_size(config.width as u32, config.height as u32);
        layer.layer_surface.set_exclusive_zone(config.exclusive_zone);
        wl_surface.commit();

        // ---- 2. Register a pending entry so Dispatch can find it ----
        let managed = ManagedSurface {
            id,
            layer,
            wl_surface,
            renderer: None,
            panels: config.panels,
        };
        self.state.register(managed);

        // ---- 3. Wait for compositor to Configure ----
        {
            // Clone the Arc outside the immutable borrow so we can pass
            // &mut self.state to wait_for_configure.
            let surface_state = {
                let surface = self.state.find_surface(id).unwrap();
                surface.layer.surface_state.clone()
            };
            self.wayland
                .wait_for_configure(&mut self.state, &surface_state);
        }

        // ---- 4. Create the Renderer with real dimensions ----
        let (w, h) = {
            let surface = self.state.find_surface(id).unwrap();
            surface.layer.dimensions()
        };

        let surface = self.state.find_surface_mut(id).unwrap();
        let renderer = Renderer::new(
            self.egl.clone(),
            &surface.wl_surface,
            w,
            h,
        );
        surface.renderer = Some(renderer);

        eprintln!("[shell] surface {id} ready ({w}x{h})");
        id
    }

    /// Run the main loop: dispatch Wayland events, then render every surface.
    /// Never returns.
    pub fn run(&mut self) {
        loop {
            // Process all pending Wayland events.
            self.wayland.dispatch(&mut self.state);

            // Render each surface that has a finished renderer.
            for entry in &self.state.surfaces {
                if let Some(ref renderer) = entry.renderer {
                    renderer.make_current();
                    let ctx = entry.render_context(&self.state);
                    renderer.render_frame(&entry.panels, &ctx);
                }
            }
        }
    }
}
