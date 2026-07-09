use gl::types::*;
use std::ffi::CString;
use std::ptr;

use crate::renderer::batch::DrawCommand;
use crate::renderer::programs::program::ShapeProgram;
use crate::renderer::programs::rect::{FillMode, GradientDirection, Mat3, RectStyle};

// ==================== QUAD CONSTANTS ====================

const QUAD_VERTS: [f32; 12] = [
    0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0,
];

// ==================== CIRCLE PROGRAM ====================

pub struct CircleProgram {
    program: GLuint,
    vao: GLuint,
    vbo: GLuint,
    surface_size_loc: GLint,
    quad_size_loc: GLint,
    rect_origin_loc: GLint,
    rect_size_loc: GLint,
    color_loc: GLint,
    border_color_loc: GLint,
    fill_mode_loc: GLint,
    gradient_direction_loc: GLint,
    gradient_stops_loc: GLint,
    gradient_color0_loc: GLint,
    gradient_color1_loc: GLint,
    gradient_color2_loc: GLint,
    gradient_color3_loc: GLint,
    softness_loc: GLint,
    no_aa_loc: GLint,
    invert_fill_loc: GLint,
    border_width_loc: GLint,
    outer_shadow_loc: GLint,
    shadow_cutout_offset_loc: GLint,
    transform_loc: GLint,
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

            let get_loc = |name: &[u8]| -> GLint {
                let loc = gl::GetUniformLocation(program, name.as_ptr() as *const GLchar);
                if loc < 0 {
                    panic!("Failed to query uniform location for: {}",
                        String::from_utf8_lossy(name));
                }
                loc
            };

            let pos_loc = gl::GetAttribLocation(program, b"a_position\0".as_ptr() as *const GLchar);
            if pos_loc < 0 {
                panic!("Failed to query attribute location for a_position");
            }

            // Upload unit quad VBO + VAO (set up once, used every frame)
            let mut vbo: GLuint = 0;
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (QUAD_VERTS.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
                QUAD_VERTS.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            let mut vao: GLuint = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::VertexAttribPointer(pos_loc as GLuint, 2, gl::FLOAT, gl::FALSE, 0, ptr::null());
            gl::EnableVertexAttribArray(pos_loc as GLuint);

            Self {
                program,
                vao,
                vbo,
                surface_size_loc: get_loc(b"u_surface_size\0"),
                quad_size_loc: get_loc(b"u_quad_size\0"),
                rect_origin_loc: get_loc(b"u_rect_origin\0"),
                rect_size_loc: get_loc(b"u_rect_size\0"),
                color_loc: get_loc(b"u_color\0"),
                border_color_loc: get_loc(b"u_border_color\0"),
                fill_mode_loc: get_loc(b"u_fill_mode\0"),
                gradient_direction_loc: get_loc(b"u_gradient_direction\0"),
                gradient_stops_loc: get_loc(b"u_gradient_stops\0"),
                gradient_color0_loc: get_loc(b"u_gradient_color0\0"),
                gradient_color1_loc: get_loc(b"u_gradient_color1\0"),
                gradient_color2_loc: get_loc(b"u_gradient_color2\0"),
                gradient_color3_loc: get_loc(b"u_gradient_color3\0"),
                softness_loc: get_loc(b"u_softness\0"),
                no_aa_loc: get_loc(b"u_no_aa\0"),
                invert_fill_loc: get_loc(b"u_invert_fill\0"),
                border_width_loc: get_loc(b"u_border_width\0"),
                outer_shadow_loc: get_loc(b"u_outer_shadow\0"),
                shadow_cutout_offset_loc: get_loc(b"u_shadow_cutout_offset\0"),
                transform_loc: get_loc(b"u_transform\0"),
            }
        }
    }

    /// Draw a single circle element.
    pub fn draw(
        &self,
        surface_width: f32,
        surface_height: f32,
        width: f32,
        height: f32,
        style: &RectStyle,
        transform: Mat3,
    ) {
        if self.program == 0 || width <= 0.0 || height <= 0.0 {
            return;
        }

        let padding = (style.border_width + style.softness + 2.0).max(2.0);
        let quad_width = width + padding * 2.0;
        let quad_height = height + padding * 2.0;
        let rect_origin = padding;
        let quad_transform = transform.multiply(&Mat3::translation(-padding, -padding));

        unsafe {
            gl::BindVertexArray(self.vao);
            gl::UseProgram(self.program);

            gl::Uniform2f(self.surface_size_loc, surface_width, surface_height);
            gl::Uniform2f(self.quad_size_loc, quad_width, quad_height);
            gl::Uniform2f(self.rect_origin_loc, rect_origin, rect_origin);
            gl::Uniform2f(self.rect_size_loc, width, height);
            gl::Uniform4f(self.color_loc, style.fill.r, style.fill.g, style.fill.b, style.fill.a);
            gl::Uniform4f(self.border_color_loc, style.border.r, style.border.g, style.border.b, style.border.a);

            let fill_mode = match style.fill_mode {
                FillMode::None => 0,
                FillMode::Solid => 1,
                FillMode::LinearGradient => 2,
            };
            gl::Uniform1i(self.fill_mode_loc, fill_mode);

            gl::Uniform2f(
                self.gradient_direction_loc,
                if style.gradient_direction == GradientDirection::Horizontal { 1.0 } else { 0.0 },
                if style.gradient_direction == GradientDirection::Vertical { 1.0 } else { 0.0 },
            );

            let s = &style.gradient_stops;
            gl::Uniform4f(self.gradient_stops_loc, s[0].position, s[1].position, s[2].position, s[3].position);
            gl::Uniform4f(self.gradient_color0_loc, s[0].color.r, s[0].color.g, s[0].color.b, s[0].color.a);
            gl::Uniform4f(self.gradient_color1_loc, s[1].color.r, s[1].color.g, s[1].color.b, s[1].color.a);
            gl::Uniform4f(self.gradient_color2_loc, s[2].color.r, s[2].color.g, s[2].color.b, s[2].color.a);
            gl::Uniform4f(self.gradient_color3_loc, s[3].color.r, s[3].color.g, s[3].color.b, s[3].color.a);

            gl::Uniform1f(self.softness_loc, style.softness);
            gl::Uniform1i(self.no_aa_loc, if style.no_aa { 1 } else { 0 });
            gl::Uniform1i(self.invert_fill_loc, if style.invert_fill { 1 } else { 0 });
            gl::Uniform1f(self.border_width_loc, style.border_width);
            gl::Uniform1i(self.outer_shadow_loc, if style.outer_shadow { 1 } else { 0 });
            gl::Uniform2f(self.shadow_cutout_offset_loc, style.shadow_cutout_offset_x, style.shadow_cutout_offset_y);

            gl::UniformMatrix3fv(self.transform_loc, 1, gl::FALSE, quad_transform.m.as_ptr());

            gl::DrawArrays(gl::TRIANGLES, 0, 6);
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
        for cmd in commands {
            self.draw(
                surface_w,
                surface_h,
                cmd.rect.w,
                cmd.rect.h,
                &cmd.style,
                Mat3::translation(cmd.rect.x, cmd.rect.y),
            );
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
            if self.vbo != 0 {
                gl::DeleteBuffers(1, &self.vbo);
            }
        }
    }
}
