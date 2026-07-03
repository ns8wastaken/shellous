pub mod programs;

use khronos_egl as egl;
use wayland_egl::WlEglSurface;
use wayland_client::{Connection, Proxy};

use gl::types::*;

use crate::renderer::programs::rect::{
    Color, CornerShape, Corners, FillMode, Mat3, RoundedRectStyle, RectProgram,
};
use crate::wayland::AppState;

// ==================== RENDERER ====================

pub struct Renderer {
    egl: egl::DynamicInstance<egl::EGL1_4>,
    egl_display: egl::Display,
    egl_surface: egl::Surface,
    rect_program: RectProgram,
    #[allow(dead_code)]
    vao: GLuint,
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

            gl::BindVertexArray(self.vao);
        }

        let surface_w = state.width as f32;
        let surface_h = state.height as f32;
        let panel_w = 260.0;
        let panel_h = surface_h;

        // ==================== PANEL BACKGROUND ====================
        let panel_style = RoundedRectStyle {
            fill: Color {
                r: 0.085,
                g: 0.095,
                b: 0.110,
                a: 1.0,
            },
            fill_mode: FillMode::Solid,
            corners: Corners {
                tl: CornerShape::Convex,
                tr: CornerShape::Concave,
                br: CornerShape::Convex,
                bl: CornerShape::Concave,
            },
            radius: Corners {
                tl: 0.0,
                tr: 12.0,
                br: 12.0,
                bl: 10.0,
            },
            softness: 0.85,
            ..Default::default()
        };

        self.rect_program.draw(
            surface_w,
            surface_h,
            panel_w,
            panel_h,
            &panel_style,
            Mat3::identity(),
        );

        // ==================== WORKSPACE INDICATORS ====================
        let start_x = 20.0;
        let spacing = 22.0;
        let dot_r = 2.5;
        let cap_r = 3.5;
        let cap_half = 5.5;
        let elem_y = surface_h * 0.5;

        for i in 0..ws_count.min(20) {
            let cx = start_x + i as f32 * spacing;

            // Stop if this element would extend past the panel edge
            if cx + cap_half + cap_r > panel_w {
                break;
            }

            if i as i32 == active_slot {
                // ---- Active: elongated capsule (rounded rect) ----
                let w = (cap_half + cap_r) * 2.0;
                let h = cap_r * 2.0;
                let rx = cx - w * 0.5;
                let ry = elem_y - h * 0.5;

                let active_style = RoundedRectStyle {
                    fill: Color {
                        r: 0.48,
                        g: 0.62,
                        b: 0.82,
                        a: 1.0,
                    },
                    fill_mode: FillMode::Solid,
                    radius: Corners {
                        tl: cap_r,
                        tr: cap_r,
                        br: cap_r,
                        bl: cap_r,
                    },
                    softness: 0.85,
                    ..Default::default()
                };

                self.rect_program.draw(
                    surface_w,
                    surface_h,
                    w,
                    h,
                    &active_style,
                    Mat3::translation(rx, ry),
                );

                // Inner highlight
                let inner_w = (cap_half * 0.6 + cap_r * 0.6) * 2.0;
                let inner_h = (cap_r * 0.6) * 2.0;
                let inner_rx = cx - inner_w * 0.5;
                let inner_ry = elem_y - inner_h * 0.5;

                let inner_style = RoundedRectStyle {
                    fill: Color {
                        r: 0.10,
                        g: 0.12,
                        b: 0.14,
                        a: 0.5,
                    },
                    fill_mode: FillMode::Solid,
                    radius: Corners {
                        tl: cap_r * 0.6,
                        tr: cap_r * 0.6,
                        br: cap_r * 0.6,
                        bl: cap_r * 0.6,
                    },
                    softness: 0.85,
                    ..Default::default()
                };

                self.rect_program.draw(
                    surface_w,
                    surface_h,
                    inner_w,
                    inner_h,
                    &inner_style,
                    Mat3::translation(inner_rx, inner_ry),
                );

                // Hover glow (semi-transparent larger capsule behind)
                if i as i32 == hover_slot {
                    let glow_w = (cap_half + cap_r + 3.0) * 2.0;
                    let glow_h = (cap_r + 3.0) * 2.0;
                    let glow_rx = cx - glow_w * 0.5;
                    let glow_ry = elem_y - glow_h * 0.5;

                    let glow_style = RoundedRectStyle {
                        fill: Color {
                            r: 0.55,
                            g: 0.70,
                            b: 0.90,
                            a: 0.12,
                        },
                        fill_mode: FillMode::Solid,
                        radius: Corners {
                            tl: cap_r + 3.0,
                            tr: cap_r + 3.0,
                            br: cap_r + 3.0,
                            bl: cap_r + 3.0,
                        },
                        softness: 1.5,
                        ..Default::default()
                    };

                    self.rect_program.draw(
                        surface_w,
                        surface_h,
                        glow_w,
                        glow_h,
                        &glow_style,
                        Mat3::translation(glow_rx, glow_ry),
                    );
                }
            } else {
                // ---- Inactive: small circle (small rounded rect) ----
                let d = dot_r * 2.0;
                let rx = cx - dot_r;
                let ry = elem_y - dot_r;

                let dot_color = if i as i32 == hover_slot {
                    Color {
                        r: 0.35,
                        g: 0.40,
                        b: 0.50,
                        a: 1.0,
                    }
                } else {
                    Color {
                        r: 0.25,
                        g: 0.28,
                        b: 0.35,
                        a: 1.0,
                    }
                };

                let dot_style = RoundedRectStyle {
                    fill: dot_color,
                    fill_mode: FillMode::Solid,
                    radius: Corners {
                        tl: dot_r,
                        tr: dot_r,
                        br: dot_r,
                        bl: dot_r,
                    },
                    softness: 0.85,
                    ..Default::default()
                };

                self.rect_program.draw(
                    surface_w,
                    surface_h,
                    d,
                    d,
                    &dot_style,
                    Mat3::translation(rx, ry),
                );

                // Hover glow
                if i as i32 == hover_slot {
                    let glow_d = (dot_r + 3.0) * 2.0;
                    let glow_rx = cx - (dot_r + 3.0);
                    let glow_ry = elem_y - (dot_r + 3.0);

                    let glow_style = RoundedRectStyle {
                        fill: Color {
                            r: 0.40,
                            g: 0.50,
                            b: 0.65,
                            a: 0.10,
                        },
                        fill_mode: FillMode::Solid,
                        radius: Corners {
                            tl: dot_r + 3.0,
                            tr: dot_r + 3.0,
                            br: dot_r + 3.0,
                            bl: dot_r + 3.0,
                        },
                        softness: 1.5,
                        ..Default::default()
                    };

                    self.rect_program.draw(
                        surface_w,
                        surface_h,
                        glow_d,
                        glow_d,
                        &glow_style,
                        Mat3::translation(glow_rx, glow_ry),
                    );
                }
            }
        }

        // ==================== BORDER STROKE ====================
        // Thin light-blue inner stroke along the bottom edge of the panel.
        let stroke_h = 1.5;
        let stroke_style = RoundedRectStyle {
            fill: Color {
                r: 0.50,
                g: 0.60,
                b: 0.78,
                a: 0.55,
            },
            fill_mode: FillMode::Solid,
            radius: Corners {
                tl: 0.0,
                tr: 0.0,
                br: 0.0,
                bl: 0.0,
            },
            softness: 0.5,
            ..Default::default()
        };

        self.rect_program.draw(
            surface_w,
            surface_h,
            panel_w,
            stroke_h,
            &stroke_style,
            Mat3::translation(0.0, 0.0), // flush with bottom of panel
        );

        // ==================== RIGHT COMPONENT (placeholder) ====================
        let right_cx = surface_w - 24.0;
        let right_w = 16.0;
        let right_h = 16.0; // capsule height = 2 * 8.0

        // Background capsule
        let right_style = RoundedRectStyle {
            fill: Color {
                r: 0.085,
                g: 0.095,
                b: 0.110,
                a: 1.0,
            },
            fill_mode: FillMode::Solid,
            radius: Corners {
                tl: 8.0,
                tr: 8.0,
                br: 8.0,
                bl: 8.0,
            },
            softness: 0.85,
            ..Default::default()
        };

        self.rect_program.draw(
            surface_w,
            surface_h,
            right_w * 2.0,
            right_h,
            &right_style,
            Mat3::translation(right_cx - right_w, elem_y - right_h * 0.5),
        );

        // Small dot inside
        let dot_d = 3.0 * 2.0;
        let dot_style = RoundedRectStyle {
            fill: Color {
                r: 0.30,
                g: 0.32,
                b: 0.40,
                a: 1.0,
            },
            fill_mode: FillMode::Solid,
            radius: Corners {
                tl: 3.0,
                tr: 3.0,
                br: 3.0,
                bl: 3.0,
            },
            softness: 0.85,
            ..Default::default()
        };

        self.rect_program.draw(
            surface_w,
            surface_h,
            dot_d,
            dot_d,
            &dot_style,
            Mat3::translation(right_cx - 3.0, elem_y - 3.0),
        );

        self.egl
            .swap_buffers(self.egl_display, self.egl_surface)
            .expect("eglSwapBuffers failed");
    }
}
