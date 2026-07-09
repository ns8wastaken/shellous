use std::cell::RefCell;
use std::ffi::CString;
use std::mem::size_of;
use std::ptr;

use gl::types::*;

use crate::renderer::batch::DrawCommand;
use crate::renderer::programs::program::ShapeProgram;

// ==================== SUPPORTING STRUCTURES ====================

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum FillMode {
    #[default]
    None,
    Solid,
    LinearGradient,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum GradientDirection {
    #[default]
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GradientStop {
    pub position: f32,
    pub color: Color,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum CornerShape {
    #[default]
    Convex,
    Concave,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Corners<T> {
    pub tl: T,
    pub tr: T,
    pub br: T,
    pub bl: T,
}

impl<T: Clone> Corners<T> {
    pub fn all(v: T) -> Self {
        Self { tl: v.clone(), tr: v.clone(), br: v.clone(), bl: v }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LogicalInset {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

#[derive(Clone, Debug, Default)]
pub struct RectStyle {
    pub fill: Color,
    pub border: Color,
    pub fill_mode: FillMode,
    pub gradient_direction: GradientDirection,
    pub gradient_stops: [GradientStop; 4],
    pub corners: Corners<CornerShape>,
    pub logical_inset: LogicalInset,
    pub radius: Corners<f32>,
    pub softness: f32,
    pub no_aa: bool,
    pub invert_fill: bool,
    pub border_width: f32,
    pub outer_shadow: bool,
    pub shadow_cutout_offset_x: f32,
    pub shadow_cutout_offset_y: f32,
}

impl RectStyle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn solid(fill: Color, radius: f32) -> Self {
        Self {
            fill,
            fill_mode: FillMode::Solid,
            radius: Corners::all(radius),
            ..Default::default()
        }
    }

    pub fn fill(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.fill = Color { r, g, b, a };
        self.fill_mode = FillMode::Solid;
        self
    }

    pub fn fill_color(mut self, color: Color) -> Self {
        self.fill = color;
        self.fill_mode = FillMode::Solid;
        self
    }

    pub fn border(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.border = Color { r, g, b, a };
        self
    }

    pub fn border_color(mut self, color: Color) -> Self {
        self.border = color;
        self
    }

    pub fn border_width(mut self, w: f32) -> Self {
        self.border_width = w;
        self
    }

    pub fn corners(
        mut self,
        tl: CornerShape,
        tr: CornerShape,
        br: CornerShape,
        bl: CornerShape,
    ) -> Self {
        self.corners = Corners { tl, tr, br, bl };
        self
    }

    pub fn radius(mut self, tl: f32, tr: f32, br: f32, bl: f32) -> Self {
        self.radius = Corners { tl, tr, br, bl };
        self
    }

    pub fn all_radius(mut self, r: f32) -> Self {
        self.radius = Corners::all(r);
        self
    }

    pub fn inset(mut self, l: f32, t: f32, r: f32, b: f32) -> Self {
        self.logical_inset = LogicalInset { left: l, top: t, right: r, bottom: b };
        self
    }

    pub fn inset_left(mut self, v: f32) -> Self {
        self.logical_inset.left = v;
        self
    }

    pub fn inset_top(mut self, v: f32) -> Self {
        self.logical_inset.top = v;
        self
    }

    pub fn inset_right(mut self, v: f32) -> Self {
        self.logical_inset.right = v;
        self
    }

    pub fn inset_bottom(mut self, v: f32) -> Self {
        self.logical_inset.bottom = v;
        self
    }

    pub fn shadow(mut self, dx: f32, dy: f32) -> Self {
        self.outer_shadow = true;
        self.shadow_cutout_offset_x = dx;
        self.shadow_cutout_offset_y = dy;
        self
    }

    pub fn softness(mut self, s: f32) -> Self {
        self.softness = s;
        self
    }
}

// ==================== PER-INSTANCE DATA ====================

#[repr(C)]
#[derive(Clone, Copy)]
struct RectInstance([f32; 60]);

// ==================== RECT PROGRAM ====================

const QUAD_VERTS: [f32; 12] = [
    0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0,
];

pub struct RectProgram {
    program: GLuint,
    vao: GLuint,
    quad_vbo: GLuint,
    instance_vbo: GLuint,
    surface_size_loc: GLint,
    instances: RefCell<Vec<RectInstance>>,
}

impl RectProgram {
    pub fn new() -> Self {
        let vert_src = include_str!("../shaders/rect.vert");
        let frag_src = include_str!("../shaders/rect.frag");

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

            // Slots 1-15: instance attributes (divisor = 1)
            let mut instance_vbo: GLuint = 0;
            gl::GenBuffers(1, &mut instance_vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, instance_vbo);

            let stride = size_of::<RectInstance>() as GLsizei;
            for slot in 1u32..=15 {
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
                instances: RefCell::new(Vec::with_capacity(64)),
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
                panic!("RectProgram shader compile error: {}", String::from_utf8_lossy(&buf));
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
                panic!("RectProgram program link error: {}", String::from_utf8_lossy(&buf));
            }
            program
        }
    }
}

impl ShapeProgram for RectProgram {
    fn draw_batch(&self, commands: &[DrawCommand], surface_w: f32, surface_h: f32) {
        if commands.is_empty() || self.program == 0 {
            return;
        }

        let corner_val = |shape| if shape == CornerShape::Concave { 1.0 } else { 0.0 };

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

            let d: [f32; 60] = [
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
                // slot 13 (a_inst12): corner_shapes
                corner_val(style.corners.tl),
                corner_val(style.corners.tr),
                corner_val(style.corners.br),
                corner_val(style.corners.bl),
                // slot 14 (a_inst13): logical_inset
                style.logical_inset.left,
                style.logical_inset.top,
                style.logical_inset.right,
                style.logical_inset.bottom,
                // slot 15 (a_inst14): radii
                style.radius.tl, style.radius.tr, style.radius.br, style.radius.bl,
            ];

            instances.push(RectInstance(d));
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
                (count * size_of::<RectInstance>()) as GLsizeiptr,
                instances.as_ptr() as *const _,
                gl::STREAM_DRAW,
            );

            gl::DrawArraysInstanced(gl::TRIANGLES, 0, 6, count as GLsizei);
        }
    }
}

impl Drop for RectProgram {
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
