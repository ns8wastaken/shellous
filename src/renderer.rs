pub mod panel;
pub mod programs;

use khronos_egl as egl;
use wayland_egl::WlEglSurface;
use wayland_client::{Connection, Proxy};

use gl::types::*;

use crate::shell_state::ShellState;
use crate::renderer::panel::Panel;
use crate::renderer::programs::rect::RectProgram;

// ==================== RENDERER ====================

pub struct Renderer {
    egl: egl::DynamicInstance<egl::EGL1_4>,
    egl_display: egl::Display,
    egl_surface: egl::Surface,
    rect_program: RectProgram,
    #[allow(dead_code)]
    vao: GLuint,
    panels: Vec<Box<dyn Panel>>,
    _wl_egl_surface: WlEglSurface,
    /// This surface's actual dimensions (from SurfaceState after configure).
    width: i32,
    height: i32,
}

impl Renderer {
    pub fn new(
        conn: &Connection,
        surface: impl wayland_client::Proxy,
        initial_width: i32,
        initial_height: i32,
        panels: Vec<Box<dyn Panel>>,
    ) -> Self {
        // ---------------- LOAD EGL ----------------
        let lib = unsafe { libloading::Library::new("libEGL.so.1") }
            .expect("unable to find libEGL.so.1");
        let egl = unsafe { egl::DynamicInstance::<egl::EGL1_4>::load_required_from(lib) }
            .expect("unable to load libEGL.so.1");

        let display_ptr = conn.display().id().as_ptr() as *mut std::ffi::c_void;
        let egl_display = unsafe { egl.get_display(display_ptr) }.expect("eglGetDisplay failed");
        egl.initialize(egl_display).expect("eglInitialize failed");
        egl.bind_api(egl::OPENGL_ES_API).expect("eglBindAPI failed");

        let config_attribs = [
            egl::SURFACE_TYPE,
            egl::WINDOW_BIT,
            egl::RENDERABLE_TYPE,
            egl::OPENGL_ES_BIT,
            egl::RED_SIZE,
            8,
            egl::GREEN_SIZE,
            8,
            egl::BLUE_SIZE,
            8,
            egl::ALPHA_SIZE,
            8,
            egl::NONE,
        ];
        let egl_config = egl
            .choose_first_config(egl_display, &config_attribs)
            .expect("eglChooseConfig failed")
            .expect("no matching EGL config found");

        let context_attribs = [
            egl::CONTEXT_MAJOR_VERSION,
            3,
            egl::CONTEXT_MINOR_VERSION,
            0,
            egl::NONE,
        ];
        let egl_context = egl
            .create_context(egl_display, egl_config, None, &context_attribs)
            .expect("eglCreateContext failed");

        let wl_egl_surface = WlEglSurface::new(surface.id(), initial_width, initial_height)
            .expect("failed to create wl_egl_surface");

        let egl_surface = unsafe {
            egl.create_window_surface(
                egl_display,
                egl_config,
                wl_egl_surface.ptr() as egl::NativeWindowType,
                None,
            )
        }
        .expect("eglCreateWindowSurface failed");

        egl.make_current(
            egl_display,
            Some(egl_surface),
            Some(egl_surface),
            Some(egl_context),
        )
        .expect("eglMakeCurrent failed");

        // Pace the render loop to vblank instead of busy-looping.
        let _ = egl.swap_interval(egl_display, 1);

        // ---------------- LOAD OPENGL ES ----------------
        gl::load_with(|s| egl.get_proc_address(s).unwrap() as *const _);

        // ---------------- CREATE RECT PROGRAM ----------------
        let rect_program = RectProgram::new();

        // ---------------- SETUP VAO ----------------
        let mut vao: GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        Self {
            egl,
            egl_display,
            egl_surface,
            rect_program,
            vao,
            panels,
            _wl_egl_surface: wl_egl_surface,
            width: initial_width,
            height: initial_height,
        }
    }

    /// Render a single frame using the current ShellState.
    /// Must be called with the EGL context current (it is after construction).
    /// Uses the Renderer's own stored dimensions (set from SurfaceState at construction).
    pub fn render_frame(&self, state: &ShellState) {
        unsafe {
            gl::Viewport(0, 0, self.width, self.height);
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::BindVertexArray(self.vao);
        }

        let surface_w = self.width as f32;
        let surface_h = self.height as f32;

        for panel in &self.panels {
            panel.draw(&self.rect_program, surface_w, surface_h, state);
        }

        self.egl
            .swap_buffers(self.egl_display, self.egl_surface)
            .expect("eglSwapBuffers failed");
    }
}
