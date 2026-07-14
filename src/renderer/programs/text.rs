use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CString;
use std::mem::size_of;
use std::ptr;
use fontdue::{Font, layout::{Layout, CoordinateSystem, TextStyle as FTextStyle}};
use gl::types::*;

use crate::renderer::types::Color;
use crate::renderer::batch::{DrawCommand, DrawParams};
use crate::renderer::programs::program::ShapeProgram;

// ==================== STYLE ====================\n
#[derive(Clone, Debug)]
pub struct TextStyle {
    pub text: String,
    pub font_size: f32,
    pub color: Color,
}

// ==================== VERTEX DATA ====================\n
#[repr(C)]
#[derive(Clone, Copy)]
pub struct TextVertex {
    pub pos: [f32; 2],
    pub uv:  [f32; 2],
}

struct CachedGlyph {
    uv_min: [f32; 2],
    uv_max: [f32; 2],
}

// Mutable state wrapped with internal mutability to protect public API signatures
struct TextAtlas {
    atlas_id: GLuint,
    atlas_width: u32,
    atlas_height: u32,
    next_x: u32,
    next_y: u32,
    max_row_height: u32,
    cache: HashMap<(char, u32), CachedGlyph>,
}

// ==================== TEXT PROGRAM ====================\n
pub struct TextProgram {
    program: GLuint,
    vao: GLuint,
    vbo: GLuint,
    surface_size_loc: GLint,
    text_color_loc: GLint,
    atlas: RefCell<TextAtlas>,
    font: Font,
}

impl TextProgram {
    pub fn new(font_bytes: &[u8], atlas_w: u32, atlas_h: u32) -> Self {
        let font = Font::from_bytes(font_bytes, fontdue::FontSettings::default())
            .expect("failed to parse interface font");

        let vert_src = include_str!("../shaders/text.vert");
        let frag_src = include_str!("../shaders/text.frag");

        let mut atlas_id: GLuint = 0;
        let mut vao = 0;
        let mut vbo = 0;

        unsafe {
            // Setup Text Program GL state
            let vs = gl::CreateShader(gl::VERTEX_SHADER);
            let c_vert = CString::new(vert_src).unwrap();
            gl::ShaderSource(vs, 1, &c_vert.as_ptr(), ptr::null());
            gl::CompileShader(vs);

            let fs = gl::CreateShader(gl::FRAGMENT_SHADER);
            let c_frag = CString::new(frag_src).unwrap();
            gl::ShaderSource(fs, 1, &c_frag.as_ptr(), ptr::null());
            gl::CompileShader(fs);

            let program = gl::CreateProgram();
            gl::AttachShader(program, vs);
            gl::AttachShader(program, fs);
            gl::LinkProgram(program);
            gl::DeleteShader(vs);
            gl::DeleteShader(fs);

            let surface_size_loc = gl::GetUniformLocation(program, b"u_surface_size\0".as_ptr() as *const GLchar);
            let text_color_loc = gl::GetUniformLocation(program, b"u_text_color\0".as_ptr() as *const GLchar);

            // Generate Texture Atlas
            gl::GenTextures(1, &mut atlas_id);
            gl::BindTexture(gl::TEXTURE_2D, atlas_id);
            gl::TexImage2D(
                gl::TEXTURE_2D, 0, gl::R8 as i32,
                atlas_w as i32, atlas_h as i32,
                0, gl::RED, gl::UNSIGNED_BYTE, ptr::null()
            );
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

            // Build standard Dynamic VAO Layout
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);
            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, size_of::<TextVertex>() as GLsizei, ptr::null());
            gl::EnableVertexAttribArray(0);

            gl::VertexAttribPointer(1, 2, gl::FLOAT, gl::FALSE, size_of::<TextVertex>() as GLsizei, (2 * size_of::<f32>()) as *const _);
            gl::EnableVertexAttribArray(1);

            gl::BindVertexArray(0);

            Self {
                program, vao, vbo, surface_size_loc, text_color_loc, font,
                atlas: RefCell::new(TextAtlas {
                    atlas_id, atlas_width: atlas_w, atlas_height: atlas_h,
                    next_x: 2, next_y: 2, max_row_height: 0, cache: HashMap::new(),
                })
            }
        }
    }
}

impl ShapeProgram for TextProgram {
    fn draw_batch(&self, commands: &[DrawCommand], surface_w: f32, surface_h: f32) {
        if commands.is_empty() { return; }

        let mut atlas = self.atlas.borrow_mut();

        unsafe {
            gl::UseProgram(self.program);
            gl::Uniform2f(self.surface_size_loc, surface_w, surface_h);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, atlas.atlas_id);

            gl::BindVertexArray(self.vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);

            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        for cmd in commands {
            let style = match &cmd.params {
                DrawParams::Text(s) => s,
                _ => continue,
            };

            let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
            layout.append(&[&self.font], &FTextStyle::new(&style.text, style.font_size, 0));

            let mut vertices = Vec::with_capacity(layout.glyphs().len() * 6);

            for glyph in layout.glyphs() {
                let size_key = style.font_size as u32;
                let character = glyph.parent;

                let cached = if atlas.cache.contains_key(&(character, size_key)) {
                    atlas.cache.get(&(character, size_key))
                } else {
                    let (metrics, bitmap) = self.font.rasterize(character, style.font_size);
                    if metrics.width > 0 && metrics.height > 0 {
                        if atlas.next_x + metrics.width as u32 + 2 > atlas.atlas_width {
                            atlas.next_x = 2;
                            atlas.next_y += atlas.max_row_height + 2;
                            atlas.max_row_height = 0;
                        }

                        let x = atlas.next_x;
                        let y = atlas.next_y;

                        unsafe {
                            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
                            gl::TexSubImage2D(
                                gl::TEXTURE_2D, 0, x as i32, y as i32,
                                metrics.width as i32, metrics.height as i32,
                                gl::RED, gl::UNSIGNED_BYTE, bitmap.as_ptr() as *const _
                            );
                        }

                        let uv_min = [x as f32 / atlas.atlas_width as f32, y as f32 / atlas.atlas_height as f32];
                        let uv_max = [
                            (x + metrics.width as u32) as f32 / atlas.atlas_width as f32,
                            (y + metrics.height as u32) as f32 / atlas.atlas_height as f32,
                        ];

                        atlas.next_x += metrics.width as u32 + 2;
                        if metrics.height as u32 > atlas.max_row_height {
                            atlas.max_row_height = metrics.height as u32;
                        }

                        atlas.cache.insert((character, size_key), CachedGlyph { uv_min, uv_max });
                        atlas.cache.get(&(character, size_key))
                    } else {
                        None
                    }
                };

                if let Some(cached) = cached {
                    let x0 = cmd.rect.x + glyph.x;
                    let y0 = cmd.rect.y + glyph.y;
                    let x1 = x0 + glyph.width as f32;
                    let y1 = y0 + glyph.height as f32;

                    let u0 = cached.uv_min[0];
                    let v0 = cached.uv_min[1];
                    let u1 = cached.uv_max[0];
                    let v1 = cached.uv_max[1];

                    // Quad generation
                    vertices.push(TextVertex { pos: [x0, y0], uv: [u0, v0] });
                    vertices.push(TextVertex { pos: [x1, y0], uv: [u1, v0] });
                    vertices.push(TextVertex { pos: [x0, y1], uv: [u0, v1] });

                    vertices.push(TextVertex { pos: [x1, y0], uv: [u1, v0] });
                    vertices.push(TextVertex { pos: [x1, y1], uv: [u1, v1] });
                    vertices.push(TextVertex { pos: [x0, y1], uv: [u0, v1] });
                }
            }

            if vertices.is_empty() { continue; }

            unsafe {
                gl::Uniform4f(self.text_color_loc, style.color.r, style.color.g, style.color.b, style.color.a);
                gl::BufferData(
                    gl::ARRAY_BUFFER,
                    (vertices.len() * size_of::<TextVertex>()) as GLsizeiptr,
                    vertices.as_ptr() as *const _,
                    gl::STREAM_DRAW,
                );
                gl::DrawArrays(gl::TRIANGLES, 0, vertices.len() as GLsizei);
            }
        }
    }
}

impl Drop for TextProgram {
    fn drop(&mut self) {
        unsafe {
            if self.program != 0 { gl::DeleteProgram(self.program); }
            if self.vao != 0 { gl::DeleteVertexArrays(1, &self.vao); }
            if self.vbo != 0 { gl::DeleteBuffers(1, &self.vbo); }
            let atlas = self.atlas.borrow();
            if atlas.atlas_id != 0 { gl::DeleteTextures(1, &atlas.atlas_id); }
        }
    }
}
