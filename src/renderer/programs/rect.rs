use gl::types::*;
use std::ffi::CString;
use std::ptr;

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

#[derive(Clone, Copy, Debug, Default)]
pub struct LogicalInset {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Mat3 {
    pub m: [f32; 9],
}

impl Mat3 {
    pub fn identity() -> Self {
        Self {
            m: [
                1.0, 0.0, 0.0,
                0.0, 1.0, 0.0,
                0.0, 0.0, 1.0,
            ],
        }
    }

    pub fn translation(x: f32, y: f32) -> Self {
        Self {
            m: [
                1.0, 0.0, 0.0,
                0.0, 1.0, 0.0,
                x,   y,   1.0,
            ],
        }
    }

    pub fn multiply(&self, other: &Self) -> Self {
        let mut out = [0.0; 9];
        for i in 0..3 {
            for j in 0..3 {
                out[i * 3 + j] = self.m[i * 3] * other.m[j]
                    + self.m[i * 3 + 1] * other.m[3 + j]
                    + self.m[i * 3 + 2] * other.m[6 + j];
            }
        }
        Self { m: out }
    }
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
    pub shadow_exclusion: bool,
    pub shadow_exclusion_offset_x: f32,
    pub shadow_exclusion_offset_y: f32,
    pub shadow_exclusion_width: f32,
    pub shadow_exclusion_height: f32,
    pub shadow_exclusion_corners: Corners<CornerShape>,
    pub shadow_exclusion_logical_inset: LogicalInset,
    pub shadow_exclusion_radius: Corners<f32>,
}

// ==================== RECT PROGRAM ====================

const QUAD_VERTS: [f32; 12] = [
    0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0,
];

pub struct RectProgram {
    program: GLuint,
    vbo: GLuint,
    pos_loc: GLint,
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
    corner_shapes_loc: GLint,
    logical_inset_loc: GLint,
    radii_loc: GLint,
    softness_loc: GLint,
    no_aa_loc: GLint,
    invert_fill_loc: GLint,
    border_width_loc: GLint,
    outer_shadow_loc: GLint,
    shadow_cutout_offset_loc: GLint,
    shadow_exclusion_loc: GLint,
    shadow_exclusion_offset_loc: GLint,
    shadow_exclusion_size_loc: GLint,
    shadow_exclusion_corner_shapes_loc: GLint,
    shadow_exclusion_logical_inset_loc: GLint,
    shadow_exclusion_radii_loc: GLint,
    transform_loc: GLint,
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

            let get_loc = |name: &[u8]| -> GLint {
                let loc = gl::GetUniformLocation(program, name.as_ptr() as *const GLchar);
                if loc < 0 {
                    panic!("Failed to query uniform location for: {}", String::from_utf8_lossy(name));
                }
                loc
            };

            let pos_loc = gl::GetAttribLocation(program, b"a_position\0".as_ptr() as *const GLchar);
            if pos_loc < 0 {
                panic!("Failed to query attribute location for a_position");
            }

            // Upload unit quad VBO (required for GLES 3.0 — no client-side arrays)
            let mut vbo: GLuint = 0;
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (QUAD_VERTS.len() * std::mem::size_of::<f32>()) as GLsizeiptr,
                QUAD_VERTS.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            Self {
                program,
                vbo,
                pos_loc,
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
                corner_shapes_loc: get_loc(b"u_corner_shapes\0"),
                logical_inset_loc: get_loc(b"u_logical_inset\0"),
                radii_loc: get_loc(b"u_radii\0"),
                softness_loc: get_loc(b"u_softness\0"),
                no_aa_loc: get_loc(b"u_no_aa\0"),
                invert_fill_loc: get_loc(b"u_invert_fill\0"),
                border_width_loc: get_loc(b"u_border_width\0"),
                outer_shadow_loc: get_loc(b"u_outer_shadow\0"),
                shadow_cutout_offset_loc: get_loc(b"u_shadow_cutout_offset\0"),
                shadow_exclusion_loc: get_loc(b"u_shadow_exclusion\0"),
                shadow_exclusion_offset_loc: get_loc(b"u_shadow_exclusion_offset\0"),
                shadow_exclusion_size_loc: get_loc(b"u_shadow_exclusion_size\0"),
                shadow_exclusion_corner_shapes_loc: get_loc(b"u_shadow_exclusion_corner_shapes\0"),
                shadow_exclusion_logical_inset_loc: get_loc(b"u_shadow_exclusion_logical_inset\0"),
                shadow_exclusion_radii_loc: get_loc(b"u_shadow_exclusion_radii\0"),
                transform_loc: get_loc(b"u_transform\0"),
            }
        }
    }

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

        let corner_shape_value = |shape| if shape == CornerShape::Concave { 1.0 } else { 0.0 };

        unsafe {
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

            gl::Uniform4f(
                self.corner_shapes_loc,
                corner_shape_value(style.corners.tl),
                corner_shape_value(style.corners.tr),
                corner_shape_value(style.corners.br),
                corner_shape_value(style.corners.bl),
            );

            gl::Uniform4f(
                self.logical_inset_loc,
                style.logical_inset.left,
                style.logical_inset.top,
                style.logical_inset.right,
                style.logical_inset.bottom,
            );

            gl::Uniform4f(self.radii_loc, style.radius.tl, style.radius.tr, style.radius.br, style.radius.bl);
            gl::Uniform1f(self.softness_loc, style.softness);
            gl::Uniform1i(self.no_aa_loc, if style.no_aa { 1 } else { 0 });
            gl::Uniform1i(self.invert_fill_loc, if style.invert_fill { 1 } else { 0 });
            gl::Uniform1f(self.border_width_loc, style.border_width);
            gl::Uniform1i(self.outer_shadow_loc, if style.outer_shadow { 1 } else { 0 });
            gl::Uniform2f(self.shadow_cutout_offset_loc, style.shadow_cutout_offset_x, style.shadow_cutout_offset_y);
            gl::Uniform1i(self.shadow_exclusion_loc, if style.shadow_exclusion { 1 } else { 0 });
            gl::Uniform2f(self.shadow_exclusion_offset_loc, style.shadow_exclusion_offset_x, style.shadow_exclusion_offset_y);
            gl::Uniform2f(self.shadow_exclusion_size_loc, style.shadow_exclusion_width, style.shadow_exclusion_height);

            gl::Uniform4f(
                self.shadow_exclusion_corner_shapes_loc,
                corner_shape_value(style.shadow_exclusion_corners.tl),
                corner_shape_value(style.shadow_exclusion_corners.tr),
                corner_shape_value(style.shadow_exclusion_corners.br),
                corner_shape_value(style.shadow_exclusion_corners.bl),
            );

            gl::Uniform4f(
                self.shadow_exclusion_logical_inset_loc,
                style.shadow_exclusion_logical_inset.left,
                style.shadow_exclusion_logical_inset.top,
                style.shadow_exclusion_logical_inset.right,
                style.shadow_exclusion_logical_inset.bottom,
            );

            gl::Uniform4f(
                self.shadow_exclusion_radii_loc,
                style.shadow_exclusion_radius.tl,
                style.shadow_exclusion_radius.tr,
                style.shadow_exclusion_radius.br,
                style.shadow_exclusion_radius.bl,
            );

            gl::UniformMatrix3fv(self.transform_loc, 1, gl::FALSE, quad_transform.m.as_ptr());

            let pos_attr = self.pos_loc as GLuint;
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::VertexAttribPointer(pos_attr, 2, gl::FLOAT, gl::FALSE, 0, ptr::null());
            gl::EnableVertexAttribArray(pos_attr);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
            gl::DisableVertexAttribArray(pos_attr);
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

impl Drop for RectProgram {
    fn drop(&mut self) {
        unsafe {
            if self.program != 0 {
                gl::DeleteProgram(self.program);
            }
            if self.vbo != 0 {
                gl::DeleteBuffers(1, &self.vbo);
            }
        }
    }
}
