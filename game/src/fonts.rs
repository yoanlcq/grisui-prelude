use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::error::Error;
use std::ffi::CString;
use std::ptr;
use std::mem;
use std::slice;
use std::mem::ManuallyDrop;
use v::{Vec2, Extent2, Aabr};
use freetype_sys as ft;
use self::ft::*;
use gx;
use grx;

#[derive(Debug)]
pub struct Fonts {
    pub ft: FT_Library,
    pub basis33: ManuallyDrop<Font>,
    pub petita: ManuallyDrop<Font>,
}

#[derive(Debug)]
pub struct Font {
    pub face: FT_Face,
    pub texture: gx::Texture2D,
    pub texture_unit: grx::TextureUnit,
    pub texture_size: Extent2<usize>,
    pub glyph_info: HashMap<char, GlyphInfo>,
}

// NOTE: We store a lot of them, so I prefer to use u16 here.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct GlyphInfo {
    // NOTE: Y axis goes downwards!
    pub coords: Aabr<u16>,
    pub offset: Vec2<u16>,
    pub x_advance: u16,
}

impl Drop for Fonts {
    fn drop(&mut self) {
        let &mut Self { ref mut basis33, ref mut petita, ref mut ft } = self;
        unsafe {
            ManuallyDrop::drop(basis33);
            ManuallyDrop::drop(petita);
            FT_Done_FreeType(*ft);
        }
    }
}

impl Drop for Font {
    fn drop(&mut self) {
        unsafe {
            FT_Done_Face(self.face);
        }
    }
}

impl Font {
    pub fn from_path(
        ft: &mut FT_Library, 
        path: &Path,
        font_size: u32,
        chars: &str,
        texture_unit: grx::TextureUnit,
        tex_side: usize,
    ) -> Result<Self, String> 
    {
        let mut face: FT_Face = unsafe { mem::uninitialized() };

        let p = CString::new(path.to_str().unwrap()).unwrap();
        if unsafe { FT_New_Face(*ft, p.as_ptr(), 0, &mut face) } != 0 {
            return Err(format!("Could not open font at `{}`", path.display()));
        }
        unsafe {
            FT_Set_Pixel_Sizes(face, 0, font_size as _);
        }

        // Partly taken from https://gist.github.com/baines/b0f9e4be04ba4e6f56cab82eef5008ff

        assert!(tex_side.is_power_of_two());
        let tex_size = Extent2::new(tex_side, tex_side);
        let mut pixels: Vec<u8> = Vec::with_capacity(tex_size.w * tex_size.h);
        unsafe {
            ptr::write_bytes(pixels.as_mut_ptr(), 0, tex_size.w * tex_size.h);
            pixels.set_len(tex_size.w * tex_size.h);
        }
        let mut pen = Vec2::<usize>::zero();
        let mut glyph_info = HashMap::new();

        for c in chars.chars() {
            if unsafe { FT_Load_Char(face, c as u64, FT_LOAD_RENDER) } != 0 {
                return Err(format!("Could not load character '{}'", c));
            }
            let g = unsafe { &*(*face).glyph };
            let bmp = &g.bitmap;
            let bmp_buffer = unsafe {
                slice::from_raw_parts(bmp.buffer, (bmp.rows*bmp.pitch) as usize)
            };
            let metrics = unsafe { &(*(*face).size).metrics };

            if pen.y + (metrics.height >> 6) as usize + 1 >= tex_size.h {
                panic!("Couldn't create font atlas for `{}`: {}x{} is not large enough!", path.display(), tex_size.w, tex_size.h);
            }
            if pen.x + bmp.width as usize >= tex_size.w {
                pen.x = 0;
                pen.y += (metrics.height >> 6) as usize + 1;
            }

            for row in 0..(bmp.rows as usize) {
                for col in 0..(bmp.width as usize) {
                    let x = pen.x + col;
                    let y = pen.y + row;
                    pixels[y * tex_size.w + x] = bmp_buffer[row * (bmp.pitch as usize) + col];
                }
            }

            let old = glyph_info.insert(c, GlyphInfo {
                coords: Aabr {
                    min: pen.map(|x| x as u16),
                    max: (pen + Vec2::new(bmp.width as _, bmp.rows as _)).map(|x| x as u16),
                },
                offset: Vec2::new(g.bitmap_left as _, g.bitmap_top as _),
                x_advance: (g.advance.x >> 6) as u16,
            });
            assert!(old.is_none());

            pen.x += bmp.width as usize + 1;
        }
        let image = gx::Texture2DImage::from_greyscale_u8(&pixels, tex_size);
        grx::set_active_texture(texture_unit);
        let texture = gx::Texture2D::new(gx::Texture2DInit {
            image, 
            params_i: gx::TextureParamsI::new_clamp_to_edge_linear(),
            do_generate_mipmaps: false,
        });
        Ok(Self { face, texture, texture_unit, glyph_info, texture_size: tex_size })
    }
}

impl Fonts {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let mut ft: FT_Library = unsafe { mem::uninitialized() };

        if unsafe { FT_Init_FreeType(&mut ft) } != 0 {
            return Err("Could not initialize FreeType library".to_string());
        }

        let mut expected = vec!["basis33", "petita"];
        match fs::read_dir(path) {
            Err(e) => return Err(e.description().to_string()),
            Ok(entries) => {
                for entry in entries.filter(Result::is_ok).map(Result::unwrap) {
                    expected.retain(|s| !entry.path().ends_with(s));
                }
                if !expected.is_empty() {
                    return Err(format!("Missing font directories {:?}", &expected));
                }
            },
        };
        let chars = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
        let basis33 = {
            let mut p = path.to_path_buf();
            p.push("basis33");
            p.push("basis33.ttf");
            ManuallyDrop::new(Font::from_path(&mut ft, &p, 16, chars, grx::TextureUnit::Basis33, 256)?)
        };
        let petita = {
            let mut p = path.to_path_buf();
            p.push("petita");
            p.push("PetitaMedium.ttf");
            ManuallyDrop::new(Font::from_path(&mut ft, &p, 18, chars, grx::TextureUnit::Petita, 256)?)
        };

        Ok(Self { ft, basis33, petita })
    }
}
