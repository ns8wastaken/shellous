use gl::types::GLuint;
use khronos_egl as egl;
use libloading::Library;

use crate::renderer::batch::Shape;
use crate::renderer::programs::ProgramRegistry;
use crate::renderer::programs::rect::RectProgram;
use crate::renderer::programs::text::TextProgram;

pub struct EglState {
    pub egl: egl::DynamicInstance<egl::EGL1_4>,
    pub egl_display: egl::Display,
    pub egl_config: egl::Config,
    pub egl_context: egl::Context,
    pub programs: ProgramRegistry,
    pub vao: GLuint,
}

impl EglState {
    pub fn new(conn: &wayland_client::Connection) -> std::sync::Arc<Self> {
        let lib = unsafe { Library::new("libEGL.so.1").expect("failed to load libEGL.so.1") };
        let egl = unsafe { egl::DynamicInstance::<egl::EGL1_4>::load_required_from(lib) }
            .expect("failed to load EGL");

        let display = unsafe {
            egl.get_display(conn.backend().display_ptr() as *mut std::ffi::c_void)
        }
        .expect("eglGetDisplay failed");

        egl.initialize(display).expect("eglInitialize failed");

        let config_attribs = [
            egl::RED_SIZE, 8,
            egl::GREEN_SIZE, 8,
            egl::BLUE_SIZE, 8,
            egl::ALPHA_SIZE, 8,
            egl::RENDERABLE_TYPE,
            egl::OPENGL_ES2_BIT,
            egl::SURFACE_TYPE,
            egl::WINDOW_BIT | egl::PBUFFER_BIT,
            egl::NONE,
        ];
        let config = egl
            .choose_first_config(display, &config_attribs)
            .expect("eglChooseConfig failed")
            .expect("no EGL config");

        let context_attribs = [egl::CONTEXT_CLIENT_VERSION, 3, egl::NONE];
        let context = egl
            .create_context(display, config, None, &context_attribs)
            .expect("eglCreateContext failed");

        let pbuffer_attribs = [egl::WIDTH, 1, egl::HEIGHT, 1, egl::NONE];
        let pbuffer = egl
            .create_pbuffer_surface(display, config, &pbuffer_attribs)
            .expect("eglCreatePbufferSurface failed");

        egl.make_current(display, Some(pbuffer), Some(pbuffer), Some(context))
            .expect("eglMakeCurrent failed");

        gl::load_with(|sym| {
            egl.get_proc_address(sym)
                .map(|proc| proc as *const ())
                .unwrap_or(std::ptr::null()) as *const std::ffi::c_void
        });

        // ==================== PROGRAM REGISTRY ====================

        let mut programs = ProgramRegistry::new();

        programs.register(Shape::Rect, RectProgram::new());

        // Load a default font
        // TODO: use system font or let user load it
        let font_bytes = include_bytes!("../../Comfortaa-Font/static/Comfortaa-Medium.ttf");
        programs.register(Shape::Text, TextProgram::new(font_bytes, 1024, 1024));

        let mut vao: GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::ONE, gl::ONE_MINUS_SRC_ALPHA);
        }

        egl.destroy_surface(display, pbuffer)
            .expect("eglDestroySurface failed");

        std::sync::Arc::new(Self {
            egl,
            egl_display: display,
            egl_config: config,
            egl_context: context,
            programs,
            vao,
        })
    }
}
