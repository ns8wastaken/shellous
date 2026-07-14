use std::collections::HashMap;
use std::ptr;
use fontdue::{Font, layout::{Layout, CoordinateSystem, TextStyle}};
use gl::types::GLuint;

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

pub struct TextRenderer {
    pub atlas_id: GLuint,
    atlas_width: u32,
    atlas_height: u32,
    next_x: u32,
    next_y: u32,
    max_row_height: u32,
    cache: HashMap<(char, u32), CachedGlyph>,
}

impl TextRenderer {
    pub fn new(atlas_width: u32, atlas_height: u32) -> Self {
        let mut atlas_id: GLuint = 0;
        unsafe {
            gl::GenTextures(1, &mut atlas_id);
            gl::BindTexture(gl::TEXTURE_2D, atlas_id);

            gl::TexImage2D(
                gl::TEXTURE_2D, 0, gl::R8 as i32,
                atlas_width as i32, atlas_height as i32,
                0, gl::RED, gl::UNSIGNED_BYTE, ptr::null()
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
        }

        Self {
            atlas_id, atlas_width, atlas_height,
            next_x: 2, next_y: 2, max_row_height: 0, // Padding from edge
            cache: HashMap::new(),
        }
    }

    pub fn cache_glyph(&mut self, font: &Font, character: char, size: f32) -> Option<&CachedGlyph> {
        let size_key = size as u32;
        if self.cache.contains_key(&(character, size_key)) {
            return self.cache.get(&(character, size_key));
        }

        let (metrics, bitmap) = font.rasterize(character, size);
        if metrics.width == 0 || metrics.height == 0 {
            return None; // Whitespaces don't need pixels packed
        }

        if self.next_x + metrics.width as u32 + 2 > self.atlas_width {
            self.next_x = 2;
            self.next_y += self.max_row_height + 2;
            self.max_row_height = 0;
        }

        let x = self.next_x;
        let y = self.next_y;

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.atlas_id);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            gl::TexSubImage2D(
                gl::TEXTURE_2D, 0, x as i32, y as i32,
                metrics.width as i32, metrics.height as i32,
                gl::RED, gl::UNSIGNED_BYTE, bitmap.as_ptr() as *const _
            );
        }

        let uv_min = [x as f32 / self.atlas_width as f32, y as f32 / self.atlas_height as f32];
        let uv_max = [
            (x + metrics.width as u32) as f32 / self.atlas_width as f32,
            (y + metrics.height as u32) as f32 / self.atlas_height as f32,
        ];

        self.next_x += metrics.width as u32 + 2;
        if metrics.height as u32 > self.max_row_height {
            self.max_row_height = metrics.height as u32;
        }

        let glyph = CachedGlyph { uv_min, uv_max };
        self.cache.insert((character, size_key), glyph);
        self.cache.get(&(character, size_key))
    }

    pub fn layout_text(&mut self, font: &Font, text: &str, size: f32, start_x: f32, start_y: f32) -> Vec<TextVertex> {
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.append(&[font], &TextStyle::new(text, size, 0));

        let mut vertices = Vec::with_capacity(layout.glyphs().len() * 6);

        for glyph in layout.glyphs() {
            if let Some(cached) = self.cache_glyph(font, glyph.parent, size) {
                let x0 = start_x + glyph.x;
                let y0 = start_y + glyph.y;
                let x1 = x0 + glyph.width as f32;
                let y1 = y0 + glyph.height as f32;

                let u0 = cached.uv_min[0];
                let v0 = cached.uv_min[1];
                let u1 = cached.uv_max[0];
                let v1 = cached.uv_max[1];

                // Triangle 1
                vertices.push(TextVertex { pos: [x0, y0], uv: [u0, v0] });
                vertices.push(TextVertex { pos: [x1, y0], uv: [u1, v0] });
                vertices.push(TextVertex { pos: [x0, y1], uv: [u0, v1] });
                // Triangle 2
                vertices.push(TextVertex { pos: [x1, y0], uv: [u1, v0] });
                vertices.push(TextVertex { pos: [x1, y1], uv: [u1, v1] });
                vertices.push(TextVertex { pos: [x0, y1], uv: [u0, v1] });
            }
        }
        vertices
    }
}

impl Drop for TextRenderer {
    fn drop(&mut self) {
        unsafe {
            if self.atlas_id != 0 {
                gl::DeleteTextures(1, &self.atlas_id);
            }
        }
    }
}
