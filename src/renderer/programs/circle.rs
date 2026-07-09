use std::cell::RefCell;
use std::ffi::CString;
use std::mem::size_of;
use std::ptr;

use gl::types::*;

use crate::renderer::batch::DrawCommand;
use crate::renderer::programs::program::ShapeProgram;
use crate::renderer::programs::rect::{FillMode, GradientDirection};

// ==================== PER-INSTANCE DATA ====================

#[repr(C)]
#[derive(Clone, Copy)]
struct CircleInstance([f32; 48]);

// ==================== CIRCLE PROGRAM ====================

const QUAD_VERTS: [f32; 12] = [
    0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0,
];

pub struct CircleProgram {
    program: GLuint,
    vao: GLuint,
    quad_vbo: GLuint,
    instance_vbo: GLuint,
    surface_size_loc: GLint,
    instances: RefCell<Vec<CircleInstance>>,
}

impl CircleProgram {
    pub fn new() -> Self {
        let vert_src = include_str!("../shaders/circle.vert");
        let frag_src = include_str!("../shaders/circle.frag");

        unsafe {
            let vs = Self::compile_shader(vert_src, gl::VERTEX_SHADER);
            let fs = Self::compile_shader(frag_src, gl::FRAGMENT_SHADER);
            let program = Self::link_program(vs, fs);
            gl::DeleteShader(vs);
            gl::DeleteShader(fs);

            let loc = |name: &[u8]| -> GLint {
                let l = gl::GetUniformLocation(program, name.as_ptr() as *const GLchar);
                if l < 0 {
                    panic!("Failed to query uniform location for: {}", String::from_utf8_lossy(name));
                }
                l
            };

            let surface_size_loc = loc(b"u_surface_size\0");

            // --- Setup VAO ---
            let mut vao: GLuint = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            // Slot 0: position (base per-vertex, divisor = 0)
            let mut quad_vbo: GLuint = 0;
            gl::GenBuffers(1, &mut quad_vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, quad_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (QUAD_VERTS.len() * size_of::<f32>()) as GLsizeiptr,
                QUAD_VERTS.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, 0, ptr::null());
            gl::EnableVertexAttribArray(0);
            gl::VertexAttribDivisor(0, 0);

            // Slots 1-12: instance attributes (divisor = 1)
            let mut instance_vbo: GLuint = 0;
            gl::GenBuffers(1, &mut instance_vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, instance_vbo);

            let stride = size_of::<CircleInstance>() as GLsizei;
            for slot in 1u32..=12 {
                let offset = ((slot as usize - 1) * 4 * size_of::<f32>()) as *const _;
                gl::VertexAttribPointer(slot, 4, gl::FLOAT, gl::FALSE, stride, offset);
                gl::EnableVertexAttribArray(slot);
                gl::VertexAttribDivisor(slot, 1);
            }

            gl::BindVertexArray(0);

            Self {
                program,
                vao,
                quad_vbo,
                instance_vbo,
                surface_size_loc,
                instances: RefCell::new(Vec::with_capacity(16)),
            }
        }
    }

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
                panic!("CircleProgram shader compile error: {}", String::from_utf8_lossy(&buf));
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
                panic!("CircleProgram program link error: {}", String::from_utf8_lossy(&buf));
            }
            program
        }
    }
}

impl ShapeProgram for CircleProgram {
    fn draw_batch(&self, commands: &[DrawCommand], surface_w: f32, surface_h: f32) {
        if commands.is_empty() || self.program == 0 {
            return;
        }

        let mut instances = self.instances.borrow_mut();
        instances.clear();
        instances.reserve(commands.len());

        for cmd in commands {
            let w = cmd.rect.w;
            let h = cmd.rect.h;
            if w <= 0.0 || h <= 0.0 {
                continue;
            }
            let style = &cmd.style;

            let padding = (style.border_width + style.softness + 2.0).max(2.0);
            let quad_w = w + padding * 2.0;
            let quad_h = h + padding * 2.0;
            let tx = cmd.rect.x - padding;
            let ty = cmd.rect.y - padding;
            let ro = padding;

            let fill_mode = match style.fill_mode {
                FillMode::None => 0.0,
                FillMode::Solid => 1.0,
                FillMode::LinearGradient => 2.0,
            };
            let gdx = if style.gradient_direction == GradientDirection::Horizontal { 1.0 } else { 0.0 };
            let gdy = if style.gradient_direction == GradientDirection::Vertical { 1.0 } else { 0.0 };
            let s = &style.gradient_stops;

            let d: [f32; 48] = [
                // slot 1 (a_inst0): translation.xy, quad_size.xy
                tx, ty, quad_w, quad_h,
                // slot 2 (a_inst1): rect_origin, rect_size.xy, fill_mode
                ro, w, h, fill_mode,
                // slot 3 (a_inst2): fill
                style.fill.r, style.fill.g, style.fill.b, style.fill.a,
                // slot 4 (a_inst3): border
                style.border.r, style.border.g, style.border.b, style.border.a,
                // slot 5 (a_inst4): grad_dir.xy, softness, no_aa
                gdx, gdy, style.softness, if style.no_aa { 1.0 } else { 0.0 },
                // slot 6 (a_inst5): invert_fill, border_width, outer_shadow, shadow_cutout.x
                if style.invert_fill { 1.0 } else { 0.0 },
                style.border_width,
                if style.outer_shadow { 1.0 } else { 0.0 },
                style.shadow_cutout_offset_x,
                // slot 7 (a_inst6): shadow_cutout.y, _, _, _
                style.shadow_cutout_offset_y, 0.0, 0.0, 0.0,
                // slot 8 (a_inst7): gradient_stops
                s[0].position, s[1].position, s[2].position, s[3].position,
                // slot 9 (a_inst8): gradient_color0
                s[0].color.r, s[0].color.g, s[0].color.b, s[0].color.a,
                // slot 10 (a_inst9): gradient_color1
                s[1].color.r, s[1].color.g, s[1].color.b, s[1].color.a,
                // slot 11 (a_inst10): gradient_color2
                s[2].color.r, s[2].color.g, s[2].color.b, s[2].color.a,
                // slot 12 (a_inst11): gradient_color3
                s[3].color.r, s[3].color.g, s[3].color.b, s[3].color.a,
            ];

            instances.push(CircleInstance(d));
        }

        let count = instances.len();
        if count == 0 {
            return;
        }

        unsafe {
            gl::UseProgram(self.program);
            gl::Uniform2f(self.surface_size_loc, surface_w, surface_h);

            gl::BindVertexArray(self.vao);

            gl::BindBuffer(gl::ARRAY_BUFFER, self.instance_vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (count * size_of::<CircleInstance>()) as GLsizeiptr,
                instances.as_ptr() as *const _,
                gl::STREAM_DRAW,
            );

            gl::DrawArraysInstanced(gl::TRIANGLES, 0, 6, count as GLsizei);
        }
    }
}

impl Drop for CircleProgram {
    fn drop(&mut self) {
        unsafe {
            if self.program != 0 {
                gl::DeleteProgram(self.program);
            }
            if self.vao != 0 {
                gl::DeleteVertexArrays(1, &self.vao);
            }
            if self.quad_vbo != 0 {
                gl::DeleteBuffers(1, &self.quad_vbo);
            }
            if self.instance_vbo != 0 {
                gl::DeleteBuffers(1, &self.instance_vbo);
            }
        }
    }
}
