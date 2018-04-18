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

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub enum FontName {
    Debug,
    Talk,
}

#[derive(Debug)]
pub struct Fonts {
    pub ft: FT_Library,
    pub fonts: ManuallyDrop<HashMap<FontName, Font>>,
}

#[derive(Debug)]
pub struct Font {
    pub face: FT_Face,
    pub texture: gx::Texture2D,
    pub texture_unit: grx::TextureUnit,
    pub texture_size: Extent2<usize>,
    pub height: u16,
    pub glyph_info: HashMap<char, GlyphInfo>,
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
        unsafe {
            ManuallyDrop::drop(&mut self.fonts);
            FT_Done_FreeType(self.ft);
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



impl From<FontName> for grx::TextureUnit {
    fn from(f: FontName) -> Self {
        match f {
            FontName::Debug => grx::TextureUnit::DebugFontAtlas,
            FontName::Talk => grx::TextureUnit::TalkFontAtlas,
        }
    }
}

impl FontName {
    pub fn try_from_texture_unit(u: grx::TextureUnit) -> Option<Self> {
        match u {
            grx::TextureUnit::DebugFontAtlas => Some(FontName::Debug),
            grx::TextureUnit::TalkFontAtlas => Some(FontName::Talk),
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
        let chars = Self::all_supported_chars();
        let fonts = {
            let mut load_font = |folder, file, size, texunit, texsize| {
                let mut p = path.to_path_buf();
                p.push(folder);
                p.push(file);
                Font::from_path(&mut ft, &p, size, &chars, texunit, texsize)
            };
            let basis33 = load_font("basis33", "basis33.ttf", 16, grx::TextureUnit::DebugFontAtlas, 256)?;
            let petita = load_font("petita", "PetitaMedium.ttf", 22, grx::TextureUnit::TalkFontAtlas, 256)?;
            let mut fonts = HashMap::new();
            fonts.insert(FontName::Debug, basis33);
            fonts.insert(FontName::Talk, petita);
            fonts
        };
        let fonts = ManuallyDrop::new(fonts);

        Ok(Self { ft, fonts })
    }
    fn all_supported_chars() -> String {
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
    }
}
