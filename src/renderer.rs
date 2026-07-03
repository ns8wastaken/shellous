pub mod programs;

use std::sync::Arc;

use wayland_egl::WlEglSurface;
use wayland_client::Proxy;

use crate::shell::egl::EglState;
use crate::ui::RenderContext;

// ==================== RENDERER ====================

/// Per-surface renderer.  Owns the EGL surface (tied to one Wayland surface)
/// and a clone of the shared `EglState` so it can make its context current
/// and access the shared `RectProgram`.
pub struct Renderer {
    egl: Arc<EglState>,
    egl_surface: khronos_egl::Surface,
    width: i32,
    height: i32,
    #[allow(dead_code)]
    _wl_egl_surface: WlEglSurface,
}

impl Renderer {
    /// Create a renderer for a single Wayland surface.
    ///
    /// `egl` is the shared EGL state created once by the `Shell`.
    /// `surface` is the `WlSurface` to render into.
    /// `width` / `height` should be the compositor-assigned dimensions
    /// (read from `SurfaceState` after `wait_for_configure`).
    pub fn new(
        egl: Arc<EglState>,
        surface: &impl Proxy,
        width: i32,
        height: i32,
    ) -> Self {
        let wl_egl_surface =
            WlEglSurface::new(surface.id(), width, height)
                .expect("failed to create wl_egl_surface");

        let egl_surface = unsafe {
            egl.egl.create_window_surface(
                egl.egl_display,
                egl.egl_config,
                wl_egl_surface.ptr() as khronos_egl::NativeWindowType,
                None,
            )
        }
        .expect("eglCreateWindowSurface failed");

        // Make context current on this surface to set vsync.
        egl.egl
            .make_current(
                egl.egl_display,
                Some(egl_surface),
                Some(egl_surface),
                Some(egl.egl_context),
            )
            .expect("eglMakeCurrent failed");
        let _ = egl.egl.swap_interval(egl.egl_display, 1);

        Self {
            egl,
            egl_surface,
            width,
            height,
            _wl_egl_surface: wl_egl_surface,
        }
    }

    /// Make this surface's EGL context current.
    /// Must be called before `render_frame`.
    pub fn make_current(&self) {
        self.egl
            .egl
            .make_current(
                self.egl.egl_display,
                Some(self.egl_surface),
                Some(self.egl_surface),
                Some(self.egl.egl_context),
            )
            .expect("eglMakeCurrent failed");
    }

    /// Render a single frame for this surface.
    ///
    pub fn render_frame(&self, ctx: &RenderContext, draw: impl FnOnce()) {
        unsafe {
            gl::Viewport(0, 0, self.width, self.height);
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::BindVertexArray(self.egl.vao);
        }

        let _ = ctx;
        draw();

        self.egl
            .egl
            .swap_buffers(self.egl.egl_display, self.egl_surface)
            .expect("eglSwapBuffers failed");
    }

    pub fn rect_program(&self) -> &crate::renderer::programs::rect::RectProgram {
        &self.egl.rect_program
    }
}
