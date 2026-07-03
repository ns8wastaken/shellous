use std::sync::Arc;

use khronos_egl as egl;
use wayland_client::{Connection, Proxy};

use gl::types::*;

use crate::renderer::programs::rect::RectProgram;

// ==================== SHARED EGL STATE ====================

/// Shared EGL/GL state owned by the Shell and referenced by every Renderer.
/// Creating one context and sharing it across all surfaces avoids redundant
/// resource creation and allows sharing GL objects (programs, VBOs).
pub struct EglState {
    pub egl: egl::DynamicInstance<egl::EGL1_4>,
    pub egl_display: egl::Display,
    pub egl_context: egl::Context,
    pub egl_config: egl::Config,
    pub rect_program: RectProgram,
    pub vao: GLuint,
}

impl EglState {
    /// Initialize EGL from a Wayland connection, load GL, and create shared
    /// resources (rect program, VAO). Returns an Arc so it can be cheaply
    /// cloned into each Renderer.
    pub fn new(conn: &Connection) -> Arc<Self> {
        // ---------------- LOAD EGL ----------------
        let lib = unsafe { libloading::Library::new("libEGL.so.1") }
            .expect("unable to find libEGL.so.1");
        let egl = unsafe { egl::DynamicInstance::<egl::EGL1_4>::load_required_from(lib) }
            .expect("unable to load libEGL.so.1");

        let display_ptr = conn.display().id().as_ptr() as *mut std::ffi::c_void;
        let egl_display =
            unsafe { egl.get_display(display_ptr) }.expect("eglGetDisplay failed");
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

        // ---------------- MAKE CONTEXT CURRENT (PBuffer) ----------------
        // GL functions require a current context.  Use a tiny PBuffer since
        // we don't have a real surface yet — we only need it for shader
        // compilation and VAO setup.
        let pbuffer_attribs = [egl::WIDTH, 1, egl::HEIGHT, 1, egl::NONE];
        let pbuffer = egl
            .create_pbuffer_surface(egl_display, egl_config, &pbuffer_attribs)
            .expect("eglCreatePbufferSurface failed");
        egl.make_current(
            egl_display,
            Some(pbuffer),
            Some(pbuffer),
            Some(egl_context),
        )
        .expect("eglMakeCurrent failed");

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

        // PBuffer is no longer needed — each Renderer will call make_current
        // with its own surface before rendering.
        egl.destroy_surface(egl_display, pbuffer)
            .expect("eglDestroySurface failed");

        Arc::new(Self {
            egl,
            egl_display,
            egl_context,
            egl_config,
            rect_program,
            vao,
        })
    }
}
