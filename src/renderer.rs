use khronos_egl as egl;
use wayland_egl::WlEglSurface;
use wayland_client::{Connection, Proxy};

use gl::types::*;

use std::ffi::CString;
use std::ptr;

use crate::bar_shader;
use crate::shader;
use crate::wayland::AppState;


// ==================== OPENGL HELPERS ====================

unsafe fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    unsafe {
        let shader = gl::CreateShader(ty);
        let c_str = CString::new(src).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        let mut success: GLint = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut len: GLint = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = vec![0u8; len as usize];
            gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            panic!("shader compile error: {}", String::from_utf8_lossy(&buf));
        }

        shader
    }
}

unsafe fn link_program(vs: GLuint, fs: GLuint) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);
        gl::LinkProgram(program);

        let mut success: GLint = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        if success == 0 {
            let mut len: GLint = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = vec![0u8; len as usize];
            gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
            panic!("program link error: {}", String::from_utf8_lossy(&buf));
        }

        program
    }
}

// ==================== RENDERER ====================

pub struct Renderer {
    egl: egl::DynamicInstance<egl::EGL1_4>,
    egl_display: egl::Display,
    egl_surface: egl::Surface,
    program: GLuint,
    vao: GLuint,
    #[allow(dead_code)]
    vbo: GLuint,
    res_loc: GLint,
    count_loc: GLint,
    active_loc: GLint,
    hover_loc: GLint,
    _wl_egl_surface: WlEglSurface,
}

impl Renderer {
    pub fn new(
        conn: &Connection,
        surface: impl wayland_client::Proxy,
        initial_width: i32,
        initial_height: i32,
    ) -> Self {
        // ---------------- LOAD EGL ----------------
        let lib = unsafe { libloading::Library::new("libEGL.so.1") }
            .expect("unable to find libEGL.so.1");
        let egl = unsafe { egl::DynamicInstance::<egl::EGL1_4>::load_required_from(lib) }
            .expect("unable to load libEGL.so.1");

        let display_ptr = conn.display().id().as_ptr() as *mut std::ffi::c_void;
        let egl_display = unsafe { egl.get_display(display_ptr) }.expect("eglGetDisplay failed");
        egl.initialize(egl_display).expect("eglInitialize failed");
        egl.bind_api(egl::OPENGL_API).expect("eglBindAPI failed");

        let config_attribs = [
            egl::SURFACE_TYPE,
            egl::WINDOW_BIT,
            egl::RENDERABLE_TYPE,
            egl::OPENGL_BIT,
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
            3,
            egl::CONTEXT_OPENGL_PROFILE_MASK,
            egl::CONTEXT_OPENGL_CORE_PROFILE_BIT,
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

        // ---------------- LOAD OPENGL ----------------
        gl::load_with(|s| egl.get_proc_address(s).unwrap() as *const _);

        // ---------------- COMPILE SHADERS ----------------
        let vs = unsafe { compile_shader(shader::VERT_SRC, gl::VERTEX_SHADER) };
        let fs = unsafe { compile_shader(bar_shader::BAR_FRAG_SRC, gl::FRAGMENT_SHADER) };
        let program = unsafe { link_program(vs, fs) };
        unsafe {
            gl::DeleteShader(vs);
            gl::DeleteShader(fs);
        }

        // ---------------- SETUP GEOMETRY ----------------
        let vertices: [f32; 12] = [
            -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, -1.0, 1.0, 1.0, -1.0, 1.0,
        ];

        let mut vao: GLuint = 0;
        let mut vbo: GLuint = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(
                0,
                2,
                gl::FLOAT,
                gl::FALSE,
                2 * std::mem::size_of::<f32>() as GLsizei,
                ptr::null(),
            );
            gl::EnableVertexAttribArray(0);

            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        // ---------------- UNIFORM LOCATIONS ----------------
        let res_loc = unsafe { gl::GetUniformLocation(program, b"resolution\0".as_ptr() as _) };
        let count_loc = unsafe { gl::GetUniformLocation(program, b"ws_count\0".as_ptr() as _) };
        let active_loc = unsafe { gl::GetUniformLocation(program, b"active_slot\0".as_ptr() as _) };
        let hover_loc = unsafe { gl::GetUniformLocation(program, b"hover_slot\0".as_ptr() as _) };

        Self {
            egl,
            egl_display,
            egl_surface,
            program,
            vao,
            vbo,
            res_loc,
            count_loc,
            active_loc,
            hover_loc,
            _wl_egl_surface: wl_egl_surface,
        }
    }

    /// Render a single frame using the current AppState.
    /// Must be called with the EGL context current (it is after construction).
    pub fn render_frame(&self, state: &AppState) {
        let (ws_count, active_slot) = {
            let bar = state.bar.lock().unwrap();
            let active_slot = bar
                .workspaces
                .iter()
                .position(|w| w.id == bar.active_id)
                .map(|i| i as i32)
                .unwrap_or(-1);
            (bar.workspaces.len(), active_slot)
        };

        let hover_slot = state
            .pointer_pos
            .and_then(|(px, py)| {
                let buttons = crate::bar::button_layout(ws_count, state.height as f32);
                crate::bar::hit_test(&buttons, px as f32, py as f32)
            })
            .map(|i| i as i32)
            .unwrap_or(-1);

        unsafe {
            gl::Viewport(0, 0, state.width, state.height);
            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(self.program);
            gl::Uniform2f(self.res_loc, state.width as f32, state.height as f32);
            gl::Uniform1i(self.count_loc, ws_count.min(16) as i32);
            gl::Uniform1i(self.active_loc, active_slot);
            gl::Uniform1i(self.hover_loc, hover_slot);

            gl::BindVertexArray(self.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }

        self.egl
            .swap_buffers(self.egl_display, self.egl_surface)
            .expect("eglSwapBuffers failed");
    }
}
