use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::error::Error;
use std::ffi::CString;
use std::ptr;
use std::mem;
use std::slice;
use std::mem::ManuallyDrop;
use std::ops::Index;
use v::{Vec2, Extent2, Aabr};
use freetype_sys as ft;
use self::ft::*;
use gx;
use grx;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum FontName {
    Basis33,
    Petita,
}

impl From<FontName> for grx::TextureUnit {
    fn from(f: FontName) -> Self {
        match f {
            FontName::Basis33 => grx::TextureUnit::Basis33,
            FontName::Petita => grx::TextureUnit::Petita,
        }
    }
}

impl FontName {
    pub fn try_from_texture_unit(u: grx::TextureUnit) -> Option<Self> {
        match u {
            grx::TextureUnit::Basis33 => Some(FontName::Basis33),
            grx::TextureUnit::Petita => Some(FontName::Petita),
        }
    }
}


#[derive(Debug)]
pub struct Fonts {
    pub ft: FT_Library,
    pub basis33: ManuallyDrop<Font>,
    pub petita: ManuallyDrop<Font>,
}

impl Index<FontName> for Fonts {
    type Output = Font;
    fn index(&self, i: FontName) -> &Font {
        match i {
            FontName::Basis33 => &self.basis33,
            FontName::Petita => &self.petita,
            // _ => panic!("No such font: {:?}", i),
        }
    }
}

#[derive(Debug)]
pub struct Font {
    pub face: FT_Face,
    pub texture: gx::Texture2D,
    pub texture_unit: grx::TextureUnit,
    pub texture_size: Extent2<usize>,
    pub glyph_info: HashMap<char, GlyphInfo>,
    pub height: u16,
}

// NOTE: We store a lot of them, so I prefer to use u16 here.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct GlyphInfo {
    // NOTE: Y axis goes downwards!
    pub bounds: Aabr<u16>,
    // Horizontal position relative to the cursor, in pixels.
    // Vertical position relative to the baseline, in pixels.
    pub offset: Vec2<i16>,
    // How far to move the cursor for the next character.
    pub advance: Vec2<i16>,
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
        tex_size: usize,
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

        let metrics = unsafe { &(*(*face).size).metrics };

        // Partly taken from https://gist.github.com/baines/b0f9e4be04ba4e6f56cab82eef5008ff

        assert!(tex_size.is_power_of_two());
        let tex_size = Extent2::broadcast(tex_size);
        let mut pixels = Vec::<u8>::with_capacity(tex_size.w * tex_size.h);
        unsafe {
            ptr::write_bytes(pixels.as_mut_ptr(), 0, tex_size.w * tex_size.h);
            pixels.set_len(tex_size.w * tex_size.h);
        }
        let mut pen = Vec2::<usize>::zero();
        let mut glyph_info = HashMap::new();

        for c in chars.chars() {
            if unsafe { FT_Load_Char(face, c as u64, FT_LOAD_RENDER) } != 0 {
                unsafe { FT_Done_Face(face); }
                return Err(format!("Could not load character '{}'", c));
            }
            let g = unsafe { &*(*face).glyph };
            let bmp = &g.bitmap;
            let bmp_buffer = unsafe {
                slice::from_raw_parts(bmp.buffer, (bmp.rows*bmp.pitch) as usize)
            };

            if pen.y + (metrics.height / 64) as usize + 1 >= tex_size.h {
                unsafe { FT_Done_Face(face); }
                panic!("Couldn't create font atlas for `{}`: {}x{} is not large enough!", path.display(), tex_size.w, tex_size.h);
            }
            if pen.x + bmp.width as usize >= tex_size.w {
                pen.x = 0;
                pen.y += (metrics.height / 64) as usize + 1;
            }

            for row in 0..(bmp.rows as usize) {
                for col in 0..(bmp.width as usize) {
                    let x = pen.x + col;
                    let y = pen.y + row;
                    pixels[y * tex_size.w + x] = bmp_buffer[row * (bmp.pitch as usize) + col];
                }
            }

            let gi = GlyphInfo {
                bounds: Aabr {
                    min: pen.map(|x| x as _),
                    max: (pen + Vec2::new(bmp.width as _, bmp.rows as _)).map(|x| x as _),
                },
                offset: Vec2::new(g.bitmap_left as _, g.bitmap_top as _),
                advance: Vec2::new(g.advance.x, g.advance.y).map(|x| (x / 64) as _),
            };
            let old = glyph_info.insert(c, gi);
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
        Ok(Self {
            face, texture, texture_unit,
            glyph_info, texture_size: tex_size, height: (metrics.height / 64) as _
        })
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
        let chars = {
            // Do include space. We only care about its GlyphInfo, so it shouldn't
            // have its place in the atlas, but: deadlines!!
            let mut chars = " ".to_string();
            // All printable ASCII chars...
            for i in 33_u8..127_u8 {
                chars.push(i as char);
            }
            // Hon hon hon Baguette Au Jambon
            chars += "°éèçàù";
            chars
        };
        let basis33 = {
            let mut p = path.to_path_buf();
            p.push("basis33");
            p.push("basis33.ttf");
            ManuallyDrop::new(Font::from_path(&mut ft, &p, 16, &chars, grx::TextureUnit::Basis33, 256)?)
        };
        let petita = {
            let mut p = path.to_path_buf();
            p.push("petita");
            p.push("PetitaMedium.ttf");
            ManuallyDrop::new(Font::from_path(&mut ft, &p, 18, &chars, grx::TextureUnit::Petita, 256)?)
        };

        Ok(Self { ft, basis33, petita })
    }
}
