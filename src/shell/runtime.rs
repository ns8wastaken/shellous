use std::cell::Cell;
use std::os::unix::io::{AsFd, AsRawFd};
use std::sync::Arc;
use std::time::Instant;

use crate::components::ui::Element;
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

    pub fn run(&mut self) {
        let shell_start = Instant::now();

        // Initial data push before the first frame
        self.state.update_surfaces(&self.workspace.handle().snapshot());

        let wayland_fd = self.wayland.conn.as_fd().as_raw_fd();
        let wake_fd = self.state.compositor.wake_fd();

        let mut fds = [
            libc::pollfd {
                fd: wayland_fd,
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: wake_fd,
                events: libc::POLLIN,
                revents: 0,
            },
        ];

        loop {
            // 1. Flush outbound requests
            let _ = self.wayland.conn.flush();

            // 2. Drain already-buffered events
            while let Ok(count) = self.wayland.event_queue.dispatch_pending(&mut self.state) {
                if count == 0 {
                    break;
                }
            }

            // 3. Prepare read guard
            let read_guard = match self.wayland.event_queue.prepare_read() {
                Some(guard) => guard,
                None => continue,  // events already buffered — loop to dispatch
            };

            // 4. Compute poll timeout from state matrix
            let timeout = if self.state.any_dirty() { 0 } else { -1 };

            // 5. Block on kernel
            let poll_ret = unsafe {
                libc::poll(fds.as_mut_ptr(), fds.len() as libc::nfds_t, timeout)
            };

            if poll_ret < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() != std::io::ErrorKind::Interrupted {
                    break;
                }
                continue;
            }

            // 6a. Process Wayland socket
            if (fds[0].revents & libc::POLLIN) != 0 {
                if read_guard.read().is_ok() {
                    let _ = self.wayland.event_queue.dispatch_pending(&mut self.state);
                }
            } else {
                std::mem::drop(read_guard);
            }

            // 6b. Process eventfd wake (workspace change)
            if (fds[1].revents & libc::POLLIN) != 0 {
                let mut buf = [0u8; 8];
                unsafe {
                    libc::read(
                        wake_fd,
                        buf.as_mut_ptr() as *mut std::ffi::c_void,
                        8,
                    );
                }
                self.state.sync_workspace_snapshots();
                self.state.update_surfaces(&self.workspace.handle().snapshot());
            }

            // 7. Compute absolute time
            let absolute_time = shell_start.elapsed().as_secs_f32();

            // 8. Tick, Layout, & Render phase
            if self.state.any_dirty() {
                let still_moving = self.state.tick_animations(absolute_time);

                if still_moving {
                    let qh = self.wayland.qh();
                    for entry in &self.state.surfaces {
                        // NOTE: not checking `entry.dirty.get()` while animating
                        if entry.renderer.is_some() {
                            entry.request_frame(qh);
                        }
                    }
                }

                self.state.compute_layouts();
                self.state.render();

                for entry in &mut self.state.surfaces {
                    entry.dirty.set(false);
                }

                if !still_moving {
                    self.state.set_animating(false);
                }
            }
        }
    }
}
